# MTECH OS — kernel module build
#   make            # build dhidi ya kernel inayoendesha (kama headers zipo)
#   make KDIR=/path/to/linux   # build dhidi ya kernel tree ya MTECH (upstream/linux)
obj-m += mtech_dev.o

KDIR ?= /lib/modules/$(shell uname -r)/build
PWD  := $(shell pwd)

all:
	$(MAKE) -C $(KDIR) M=$(PWD) modules

clean:
	$(MAKE) -C $(KDIR) M=$(PWD) clean

load: all
	sudo insmod mtech_dev.ko mtech_ring_kb=64
	sudo dmesg | tail -5

unload:
	sudo rmmod mtech_dev

test: load
	cat /dev/mtech | head -5 ; true
	cat /proc/mtech_status
