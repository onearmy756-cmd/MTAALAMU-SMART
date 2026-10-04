// SPDX-License-Identifier: GPL-2.0
/*
 * MTECH OS — mtech_dev.c
 *
 * Kiunganishi cha kernel: /dev/mtech
 *  - Kprobes kwenye do_execve() na do_exit() → kila process inayoanza/kufa
 *    inaandikwa kwenye ring buffer ya kernel.
 *  - Agent (Qwen 2.5 VL 3B) na GUI husoma matukio haya kupitia /dev/mtech.
 *  - /proc/mtech_status — hali ya moduli.
 *
 * Build (ndani ya kernel tree ya MTECH):
 *   make -C <linux> M=$PWD modules
 *   sudo insmod mtech_dev.ko mtech_ring_kb=64
 *
 * Soma matukio:
 *   cat /dev/mtech          # "ts|exec|pid|comm|path"
 *   echo clear > /dev/mtech # futa ring
 */
#define pr_fmt(fmt) KBUILD_MODNAME ": " fmt

#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/init.h>
#include <linux/miscdevice.h>
#include <linux/fs.h>
#include <linux/kfifo.h>
#include <linux/kprobes.h>
#include <linux/proc_fs.h>
#include <linux/seq_file.h>
#include <linux/slab.h>
#include <linux/uaccess.h>
#include <linux/poll.h>
#include <linux/sched.h>
#include <linux/timekeeping.h>

MODULE_LICENSE("GPL");
MODULE_AUTHOR("MTECH OS");
MODULE_DESCRIPTION("MTECH OS kernel bridge: /dev/mtech event feed (exec/exit)");
MODULE_VERSION("0.1");

static unsigned int mtech_ring_kb = 64;
module_param(mtech_ring_kb, uint, 0444);
MODULE_PARM_DESC(mtech_ring_kb, "Ring buffer size (KB)");

static DEFINE_SPINLOCK(mtech_lock);
static struct kfifo mtech_fifo;
static DECLARE_WAIT_QUEUE_HEAD(mtech_waitq);
static atomic_t ev_exec = ATOMIC_INIT(0);
static atomic_t ev_exit = ATOMIC_INIT(0);
static bool fifo_ready;

static void mtech_push(const char *type, const char *detail)
{
	char line[256];
	int len;
	u64 ts = ktime_get_real_seconds();

	len = snprintf(line, sizeof(line), "%llu|%s|%d|%s|%s\n",
		       (unsigned long long)ts, type, current->pid,
		       current->comm, detail ? detail : "-");
	if (len <= 0)
		return;

	spin_lock(&mtech_lock);
	if (fifo_ready) {
		unsigned int avail = kfifo_avail(&mtech_fifo);
		if ((unsigned int)len > avail) {
			/* ring imejaa: toa mistari ya zamani */
			char drop[256];
			unsigned int need = (unsigned int)len - avail;
			unsigned int got = 0;
			while (got < need && kfifo_out(&mtech_fifo, drop, min(sizeof(drop), need)))
				got += min(sizeof(drop), need);
		}
		kfifo_in(&mtech_fifo, line, (unsigned int)len);
	}
	spin_unlock(&mtech_lock);
	wake_up_interruptible(&mtech_waitq);
}

/* ------------------------------------------------ kprobe: do_execve */
static int kp_exec_pre(struct kprobe *p, struct pt_regs *regs)
{
	const char *path = "-";
	struct filename *fn;

#ifdef CONFIG_X86_64
	fn = (struct filename *)regs->di;
#elif defined(CONFIG_ARM64)
	fn = (struct filename *)regs->regs[0];
#else
	fn = NULL;
#endif
	if (!IS_ERR_OR_NULL(fn))
		path = fn->name ? fn->name : "-";

	atomic_inc(&ev_exec);
	mtech_push("exec", path);
	return 0;
}

/* ------------------------------------------------ kprobe: do_exit */
static int kp_exit_pre(struct kprobe *p, struct pt_regs *regs)
{
	long code = 0;

#ifdef CONFIG_X86_64
	code = (long)regs->di;
#elif defined(CONFIG_ARM64)
	code = (long)regs->regs[0];
#endif
	atomic_inc(&ev_exit);
	mtech_push("exit", "0");
	return 0;
}

static struct kprobe kp_exec = { .symbol_name = "do_execve", .pre_handler = kp_exec_pre };
static struct kprobe kp_exit = { .symbol_name = "do_exit",   .pre_handler = kp_exit_pre };

/* ------------------------------------------------ /dev/mtech */
static int mtech_open(struct inode *ino, struct file *filp)
{
	return 0;
}

static ssize_t mtech_read(struct file *filp, char __user *buf, size_t count, loff_t *ppos)
{
	char *kbuf;
	unsigned int copied;
	int ret = 0;

	if (count == 0)
		return 0;
	kbuf = kmalloc(min(count, (size_t)4096), GFP_KERNEL);
	if (!kbuf)
		return -ENOMEM;

	if (!(filp->f_flags & O_NONBLOCK)) {
		ret = wait_event_interruptible(mtech_waitq,
			!kfifo_is_empty(&mtech_fifo) || !fifo_ready);
		if (ret)
			goto out_free;
	}

	spin_lock(&mtech_lock);
	if (fifo_ready)
		ret = kfifo_out(&mtech_fifo, kbuf, (unsigned int)min(count, (size_t)4096));
	else
		ret = 0;
	spin_unlock(&mtech_lock);

	copied = ret;
	if (copied == 0) {
		ret = (filp->f_flags & O_NONBLOCK) ? -EAGAIN : 0;
		goto out_free;
	}
	if (copy_to_user(buf, kbuf, copied)) {
		ret = -EFAULT;
		goto out_free;
	}
	ret = copied;
out_free:
	kfree(kbuf);
	return ret;
}

static ssize_t mtech_write(struct file *filp, const char __user *buf, size_t count, loff_t *ppos)
{
	char cmd[32];

	if (count >= sizeof(cmd))
		return -EINVAL;
	if (copy_from_user(cmd, buf, count))
		return -EFAULT;
	cmd[count] = '\0';
	if (sysfs_streq(cmd, "clear")) {
		char drop[256];
		unsigned int n;
		spin_lock(&mtech_lock);
		while ((n = kfifo_out(&mtech_fifo, drop, sizeof(drop))) > 0)
			;
		spin_unlock(&mtech_lock);
		return (ssize_t)count;
	}
	return -EINVAL;
}

static __poll_t mtech_poll(struct file *filp, poll_table *wait)
{
	__poll_t mask = 0;

	poll_wait(filp, &mtech_waitq, wait);
	spin_lock(&mtech_lock);
	if (!kfifo_is_empty(&mtech_fifo))
		mask |= EPOLLIN | EPOLLRDNORM;
	spin_unlock(&mtech_lock);
	return mask;
}

static const struct file_operations mtech_fops = {
	.owner   = THIS_MODULE,
	.open    = mtech_open,
	.read    = mtech_read,
	.write   = mtech_write,
	.poll    = mtech_poll,
	.llseek  = no_llseek,
};

static struct miscdevice mtech_misc = {
	.minor = MISC_DYNAMIC_MINOR,
	.name  = "mtech",
	.fops  = &mtech_fops,
	.mode  = 0660,
};

/* ------------------------------------------------ /proc/mtech_status */
static int mtech_status_show(struct seq_file *m, void *v)
{
	seq_printf(m, "MTECH OS kernel bridge\n");
	seq_printf(m, "source:      kprobes (do_execve, do_exit)\n");
	seq_printf(m, "events exec: %d\n", atomic_read(&ev_exec));
	seq_printf(m, "events exit: %d\n", atomic_read(&ev_exit));
	spin_lock(&mtech_lock);
	seq_printf(m, "ring:        %u / %u bytes used (param mtech_ring_kb=%u KB)\n",
		   fifo_ready ? kfifo_len(&mtech_fifo) : 0,
		   fifo_ready ? kfifo_size(&mtech_fifo) : 0, mtech_ring_kb);
	spin_unlock(&mtech_lock);
	return 0;
}

static int mtech_status_open(struct inode *ino, struct file *filp)
{
	return single_open(filp, mtech_status_show, NULL);
}

static const struct proc_ops mtech_proc_ops = {
	.proc_open    = mtech_status_open,
	.proc_read    = seq_read,
	.proc_lseek   = seq_lseek,
	.proc_release = single_release,
};

/* ------------------------------------------------ init/exit */
static int __init mtech_init(void)
{
	int ret;

	ret = kfifo_alloc(&mtech_fifo, mtech_ring_kb * 1024, GFP_KERNEL);
	if (ret) {
		pr_err("kfifo_alloc imeshindikana (%d)\n", ret);
		return ret;
	}
	fifo_ready = true;

	ret = misc_register(&mtech_misc);
	if (ret) {
		pr_err("misc_register imeshindikana (%d)\n", ret);
		goto err_fifo;
	}

	ret = register_kprobe(&kp_exec);
	if (ret)
		goto err_misc;
	ret = register_kprobe(&kp_exit);
	if (ret)
		goto err_exec;

	proc_create("mtech_status", 0444, NULL, &mtech_proc_ops);
	pr_info("/dev/mtech tayari — MTECH OS inaona kernel\n");
	return 0;

err_exec:
	unregister_kprobe(&kp_exec);
err_misc:
	misc_deregister(&mtech_misc);
err_fifo:
	fifo_ready = false;
	kfifo_free(&mtech_fifo);
	return ret;
}

static void __exit mtech_exit(void)
{
	remove_proc_entry("mtech_status", NULL);
	unregister_kprobe(&kp_exit);
	unregister_kprobe(&kp_exec);
	misc_deregister(&mtech_misc);
	fifo_ready = false;
	kfifo_free(&mtech_fifo);
	pr_info("/dev/mtech imefungwa\n");
}

module_init(mtech_init);
module_exit(mtech_exit);
