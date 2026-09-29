# Grounded Reasoner — fikra + calculus + R + Rust (bila hallucination)

## Kanuni

1. **Kila dai lina chanzo** (`problems`, `devices`, `sysprobe`, `formula`, `calculus`).
2. **Hakuna kubuni** brand/model/dalili zisizokuwepo kwenye ujumbe au database.
3. **Hesabu** ni deterministic: linear load index, partial derivatives, softmax, logistic risk, central difference.
4. **Majibu marefu** kwa Kiswahili: thinking → evidence → calculus → mapendekezo → mipaka.

## Calculus (Rust)

```
P = 0.4·CPU + 0.35·RAM + 0.25·Disk
∂P/∂CPU = 0.4,  ∂P/∂RAM = 0.35,  ∂P/∂Disk = 0.25
r(P) = 1/(1+e^(-(P-50)/10))   # logistic risk
r'(P) ≈ (r(P+h)-r(P-h))/(2h) # numeric derivative
softmax(CPU,RAM,Disk)          # priority weights
```

## CLI

```bash
cd engine-rust && cargo build --release
./target/release/mtaalamu reason --msg "kompyuta polepole virus"
./target/release/mtaalamu agentic --msg "CPU juu" --approve
```

## R

```r
source("web-r/R/grounded.R")
grounded_summary("TV haiwaki", cpu = 40, mem = 60, disk = 70)
```

## Si AGI

Orchestrator + knowledge + probe + hesabu. Hardware = mwongozo kwa binadamu.
