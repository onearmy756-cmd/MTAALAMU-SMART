// Command akilibridge ni daraja la HTTP kwa daemon ya akilid (Rust).
//
// Hutoa API mbili:
//   - /akili/*        — API ya asili ya MTECH OS (chat, remember, recall, digest)
//   - /v1/*           — API inayolingana na OpenAI (hivyo R/Shiny, apps, na
//     zana nyingine zinaweza kuzungumza bila mabadiliko)
package main

import (
	"context"
	"encoding/json"
	"flag"
	"fmt"
	"log"
	"net/http"
	"os"
	"strings"
	"time"

	"mtechos/akilibridge/internal/akili"
)

type server struct {
	client *akili.Client
	model  string
	start  time.Time
}

func main() {
	var (
		addr    = flag.String("addr", ":8099", "anwani ya HTTP")
		binary  = flag.String("akilid", envOr("AKILID_BIN", "akilid"), "njia ya akilid")
		model   = flag.String("model", os.Getenv("AKILI_MODEL"), "njia ya modeli")
		dtype   = flag.String("dtype", envOr("AKILI_DTYPE", "f16"), "f16 | f32")
		memory  = flag.String("memory", envOr("AKILI_MEMORY", "data/kumbukumbu.jsonl"), "faili ya kumbukumbu")
		timeout = flag.Duration("start-timeout", 10*time.Minute, "muda wa kusubiri modeli ipakiwe")
	)
	flag.Parse()

	if *model == "" {
		log.Fatal("weka --model au AKILI_MODEL (njia ya Qwen2.5-VL)")
	}
	log.Printf("inaanzisha akilid: %s (%s, %s)", *model, *dtype, *binary)

	ctx, cancel := context.WithTimeout(context.Background(), *timeout)
	defer cancel()
	client, err := akili.Start(ctx, akili.Options{
		Binary: *binary, Model: *model, DType: *dtype, Memory: *memory,
	})
	if err != nil {
		log.Fatalf("kosa la kuanzisha akilid: %v", err)
	}
	defer client.Close()

	s := &server{client: client, model: *model, start: time.Now()}
	mux := http.NewServeMux()
	mux.HandleFunc("/healthz", s.handleHealth)
	mux.HandleFunc("/akili/chat", s.handleChat)
	mux.HandleFunc("/akili/remember", s.handleRemember)
	mux.HandleFunc("/akili/recall", s.handleRecall)
	mux.HandleFunc("/akili/digest", s.handleDigest)
	mux.HandleFunc("/v1/models", s.handleModels)
	mux.HandleFunc("/v1/chat/completions", s.handleOpenAIChat)

	log.Printf("akilibridge inasikiliza %s", *addr)
	srv := &http.Server{Addr: *addr, Handler: logRequests(mux), ReadHeaderTimeout: 30 * time.Second}
	if err := srv.ListenAndServe(); err != nil {
		log.Fatal(err)
	}
}

func envOr(k, d string) string {
	if v := os.Getenv(k); v != "" {
		return v
	}
	return d
}

func writeJSON(w http.ResponseWriter, code int, v any) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(code)
	_ = json.NewEncoder(w).Encode(v)
}

func writeErr(w http.ResponseWriter, code int, format string, args ...any) {
	writeJSON(w, code, map[string]any{"ok": false, "error": fmt.Sprintf(format, args...)})
}

func logRequests(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		start := time.Now()
		next.ServeHTTP(w, r)
		log.Printf("%s %s (%s)", r.Method, r.URL.Path, time.Since(start).Round(time.Millisecond))
	})
}

// ---------- API ya asili ----------

func (s *server) handleHealth(w http.ResponseWriter, _ *http.Request) {
	resp, err := s.client.Health()
	if err != nil {
		writeErr(w, http.StatusServiceUnavailable, "%v", err)
		return
	}
	writeJSON(w, http.StatusOK, map[string]any{
		"ok":       true,
		"uptime_s": time.Since(s.start).Seconds(),
		"engine":   json.RawMessage(resp.Info),
	})
}

func (s *server) handleChat(w http.ResponseWriter, r *http.Request) {
	var req akili.Request
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeErr(w, http.StatusBadRequest, "JSON si sahihi: %v", err)
		return
	}
	if len(req.Messages) == 0 {
		writeErr(w, http.StatusBadRequest, "`messages` ni tupu")
		return
	}
	req.Op = "chat"
	resp, err := s.client.Do(req)
	if err != nil {
		writeErr(w, http.StatusBadGateway, "%v", err)
		return
	}
	writeJSON(w, http.StatusOK, resp)
}

func (s *server) handleRemember(w http.ResponseWriter, r *http.Request) {
	var req akili.Request
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeErr(w, http.StatusBadRequest, "JSON si sahihi: %v", err)
		return
	}
	if strings.TrimSpace(req.Text) == "" {
		writeErr(w, http.StatusBadRequest, "`text` ni tupu")
		return
	}
	req.Op = "remember"
	resp, err := s.client.Do(req)
	if err != nil {
		writeErr(w, http.StatusBadGateway, "%v", err)
		return
	}
	writeJSON(w, http.StatusOK, resp)
}

func (s *server) handleRecall(w http.ResponseWriter, r *http.Request) {
	q := r.URL.Query().Get("query")
	if q == "" {
		var req akili.Request
		_ = json.NewDecoder(r.Body).Decode(&req)
		q = req.Query
	}
	limit := 8
	if l := r.URL.Query().Get("limit"); l != "" {
		fmt.Sscanf(l, "%d", &limit)
	}
	resp, err := s.client.Do(akili.Request{Op: "recall", Query: q, Limit: limit})
	if err != nil {
		writeErr(w, http.StatusBadGateway, "%v", err)
		return
	}
	writeJSON(w, http.StatusOK, resp)
}

func (s *server) handleDigest(w http.ResponseWriter, r *http.Request) {
	limit := 2000
	if l := r.URL.Query().Get("limit"); l != "" {
		fmt.Sscanf(l, "%d", &limit)
	}
	resp, err := s.client.Do(akili.Request{Op: "digest", Limit: limit})
	if err != nil {
		writeErr(w, http.StatusBadGateway, "%v", err)
		return
	}
	writeJSON(w, http.StatusOK, resp)
}

// ---------- API inayolingana na OpenAI ----------

func (s *server) handleModels(w http.ResponseWriter, _ *http.Request) {
	writeJSON(w, http.StatusOK, map[string]any{
		"object": "list",
		"data": []map[string]any{
			{"id": "mtaalamu", "object": "model", "owned_by": "mtechos", "created": s.start.Unix()},
			{"id": "qwen2.5-vl", "object": "model", "owned_by": "mtechos", "created": s.start.Unix()},
		},
	})
}

type openAIMessage struct {
	Role    string          `json:"role"`
	Content json.RawMessage `json:"content"`
}

type openAIRequest struct {
	Model       string          `json:"model"`
	Messages    []openAIMessage `json:"messages"`
	MaxTokens   int             `json:"max_tokens"`
	Temperature float64         `json:"temperature"`
	TopP        float64         `json:"top_p"`
	Stream      bool            `json:"stream"`
}

func (s *server) handleOpenAIChat(w http.ResponseWriter, r *http.Request) {
	var req openAIRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeErr(w, http.StatusBadRequest, "JSON si sahihi: %v", err)
		return
	}
	if req.Stream {
		writeErr(w, http.StatusNotImplemented, "stream haitumiki bado")
		return
	}

	var (
		msgs   []akili.Msg
		images []string
		system string
	)
	for _, m := range req.Messages {
		text, imgs := parseContent(m.Content)
		if m.Role == "system" {
			system = text
			continue
		}
		msgs = append(msgs, akili.Msg{Role: m.Role, Content: text})
		images = append(images, imgs...)
	}
	if len(msgs) == 0 {
		writeErr(w, http.StatusBadRequest, "`messages` ni tupu")
		return
	}

	maxTokens := req.MaxTokens
	if maxTokens == 0 {
		maxTokens = 512
	}
	resp, err := s.client.Do(akili.Request{
		Op: "chat", System: system, Messages: msgs, Images: images,
		MaxTokens: maxTokens, Temp: req.Temperature, TopP: req.TopP,
	})
	if err != nil {
		writeErr(w, http.StatusBadGateway, "%v", err)
		return
	}

	writeJSON(w, http.StatusOK, map[string]any{
		"id":      fmt.Sprintf("chatcmpl-%d", time.Now().UnixNano()),
		"object":  "chat.completion",
		"created": time.Now().Unix(),
		"model":   "mtaalamu",
		"choices": []map[string]any{{
			"index":         0,
			"finish_reason": "stop",
			"message":       map[string]any{"role": "assistant", "content": resp.Text},
		}},
		"usage": map[string]any{
			"prompt_tokens":     resp.PromptTokens,
			"completion_tokens": resp.GeneratedTokens,
			"total_tokens":      resp.PromptTokens + resp.GeneratedTokens,
		},
	})
}

// parseContent hubadilisha `content` (string au orodha ya parts) kuwa maandishi + picha.
func parseContent(raw json.RawMessage) (string, []string) {
	if len(raw) == 0 {
		return "", nil
	}
	var s string
	if err := json.Unmarshal(raw, &s); err == nil {
		return s, nil
	}
	var parts []struct {
		Type     string `json:"type"`
		Text     string `json:"text"`
		ImageURL struct {
			URL string `json:"url"`
		} `json:"image_url"`
	}
	if err := json.Unmarshal(raw, &parts); err != nil {
		return "", nil
	}
	var (
		sb   strings.Builder
		imgs []string
	)
	for _, p := range parts {
		switch p.Type {
		case "text":
			sb.WriteString(p.Text)
		case "image_url":
			imgs = append(imgs, normalizeImageRef(p.ImageURL.URL))
		}
	}
	return sb.String(), imgs
}

// normalizeImageRef hubadilisha data-URL kuwa `base64:...` (au huacha njia ya faili).
func normalizeImageRef(u string) string {
	if rest, ok := strings.CutPrefix(u, "data:"); ok {
		if i := strings.Index(rest, ";base64,"); i >= 0 {
			return "base64:" + rest[i+len(";base64,"):]
		}
	}
	return u
}
