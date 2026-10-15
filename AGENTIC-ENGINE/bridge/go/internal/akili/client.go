// Package akili ni kiungo cha Go kwa daemon ya akilid (Rust).
//
// akilid huzungumza JSON-lines kwenye stdin/stdout. Client hii huiendesha
// mara moja na kutumia mutex kuhakikisha ombi moja kwa wakati mmoja.
package akili

import (
	"bufio"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"os/exec"
	"sync"
	"time"
)

// Request ni ombi moja la JSON kwa akilid.
type Request struct {
	ID         any      `json:"id,omitempty"`
	Op         string   `json:"op"`
	System     string   `json:"system,omitempty"`
	Messages   []Msg    `json:"messages,omitempty"`
	Images     []string `json:"images,omitempty"`
	MaxTokens  int      `json:"max_tokens,omitempty"`
	Temp       float64  `json:"temperature,omitempty"`
	TopP       float64  `json:"top_p,omitempty"`
	RepPenalty float64  `json:"repetition_penalty,omitempty"`
	Kind       string   `json:"kind,omitempty"`
	Text       string   `json:"text,omitempty"`
	Tags       []string `json:"tags,omitempty"`
	Salience   float64  `json:"salience,omitempty"`
	Query      string   `json:"query,omitempty"`
	Limit      int      `json:"limit,omitempty"`
	Remember   bool     `json:"remember,omitempty"`
}

// Msg ni ujumbe mmoja wa mazungumzo.
type Msg struct {
	Role    string   `json:"role"`
	Content string   `json:"content"`
	Images  []string `json:"images,omitempty"`
}

// Memory ni kumbukumbu moja iliyorejeshwa.
type Memory struct {
	ID        uint64   `json:"id"`
	Kind      string   `json:"kind"`
	Text      string   `json:"text"`
	Tags      []string `json:"tags"`
	Salience  float32  `json:"salience"`
	CreatedAt string   `json:"created_at"`
	Score     float32  `json:"score,omitempty"`
}

// Response ni jibu la akilid.
type Response struct {
	ID              any             `json:"id"`
	OK              bool            `json:"ok"`
	Error           string          `json:"error,omitempty"`
	Text            string          `json:"text,omitempty"`
	PromptTokens    int             `json:"prompt_tokens,omitempty"`
	GeneratedTokens int             `json:"generated_tokens,omitempty"`
	Stopped         bool            `json:"stopped,omitempty"`
	MS              uint64          `json:"ms,omitempty"`
	Memories        []Memory        `json:"memories,omitempty"`
	MemoryID        uint64          `json:"memory_id,omitempty"`
	Info            json.RawMessage `json:"info,omitempty"`
}

// ErrClosed hurudishwa ombi likifanywa baada ya daemon kufungwa.
var ErrClosed = errors.New("akili: daemon imefungwa")

// Client huhifadhi process ya akilid.
type Client struct {
	cmd    *exec.Cmd
	stdin  io.WriteCloser
	stdout *bufio.Reader
	mu     sync.Mutex
	seq    int
	closed bool
}

// Options ni mipangilio ya kuanzisha daemon.
type Options struct {
	Binary string // njia ya akilid (default: "akilid")
	Model  string // njia ya modeli ya Qwen2.5-VL
	DType  string // f16 | f32
	Memory string // faili ya kumbukumbu (JSONL)
}

// Start huanzisha akilid na kusubiri iwe tayari.
func Start(ctx context.Context, opts Options) (*Client, error) {
	if opts.Binary == "" {
		opts.Binary = "akilid"
	}
	if opts.DType == "" {
		opts.DType = "f16"
	}
	if opts.Model == "" {
		return nil, errors.New("akili: Options.Model inahitajika")
	}
	args := []string{"serve", "--model", opts.Model, "--dtype", opts.DType}
	if opts.Memory != "" {
		args = append(args, "--memory", opts.Memory)
	}
	cmd := exec.CommandContext(ctx, opts.Binary, args...)
	stdin, err := cmd.StdinPipe()
	if err != nil {
		return nil, err
	}
	stdout, err := cmd.StdoutPipe()
	if err != nil {
		return nil, err
	}
	cmd.Stderr = nil
	if err := cmd.Start(); err != nil {
		return nil, fmt.Errorf("akili: kuanzisha akilid: %w", err)
	}
	c := &Client{cmd: cmd, stdin: stdin, stdout: bufio.NewReaderSize(stdout, 1<<20)}
	return c, nil
}

// Close husimamisha daemon.
func (c *Client) Close() error {
	c.mu.Lock()
	defer c.mu.Unlock()
	if c.closed {
		return nil
	}
	c.closed = true
	_ = c.stdin.Close()
	if c.cmd.Process != nil {
		_ = c.cmd.Process.Kill()
	}
	return c.cmd.Wait()
}

// Do hutuma ombi na hurudisha jibu. Ni salama kutumia kutoka goroutine nyingi.
func (c *Client) Do(req Request) (*Response, error) {
	c.mu.Lock()
	defer c.mu.Unlock()
	if c.closed {
		return nil, ErrClosed
	}
	c.seq++
	req.ID = c.seq
	raw, err := json.Marshal(req)
	if err != nil {
		return nil, err
	}
	if _, err := c.stdin.Write(append(raw, '\n')); err != nil {
		return nil, err
	}
	line, err := c.stdout.ReadBytes('\n')
	if err != nil {
		return nil, fmt.Errorf("akili: kusoma jibu: %w", err)
	}
	var resp Response
	if err := json.Unmarshal(line, &resp); err != nil {
		return nil, fmt.Errorf("akili: jibu si JSON sahihi: %w", err)
	}
	if !resp.OK {
		return &resp, fmt.Errorf("akili: %s", resp.Error)
	}
	return &resp, nil
}

// Chat hutuma mazungumzo na hurudisha maandishi.
func (c *Client) Chat(system string, messages []Msg, images []string, maxTokens int) (string, *Response, error) {
	resp, err := c.Do(Request{
		Op:        "chat",
		System:    system,
		Messages:  messages,
		Images:    images,
		MaxTokens: maxTokens,
	})
	if err != nil {
		return "", resp, err
	}
	return resp.Text, resp, nil
}

// Remember huhifadhi kumbukumbu.
func (c *Client) Remember(text string, tags []string) (uint64, error) {
	resp, err := c.Do(Request{Op: "remember", Text: text, Tags: tags, Kind: "ukweli", Salience: 1.0})
	if err != nil {
		return 0, err
	}
	return resp.MemoryID, nil
}

// Recall hutafuta kumbukumbu.
func (c *Client) Recall(query string, limit int) ([]Memory, error) {
	resp, err := c.Do(Request{Op: "recall", Query: query, Limit: limit})
	if err != nil {
		return nil, err
	}
	return resp.Memories, nil
}

// Health hupima afya ya injini.
func (c *Client) Health() (*Response, error) {
	return c.Do(Request{Op: "health"})
}

// Ping hupima muda wa jibu kwa ombi la afya.
func (c *Client) Ping() (time.Duration, error) {
	start := time.Now()
	_, err := c.Health()
	return time.Since(start), err
}
