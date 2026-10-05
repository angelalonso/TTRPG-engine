# Negotiation Simulation Program Specification

## Overview
This document outlines the system logic, parameters, and algorithms required to simulate realistic human-like negotiations between a **Sponsor** and a **Sponsored Party**.

In this scenario:
* Both parties refer to a public **Median Benchmark Value ($M$)**.
* The **Sponsor** wants to pay below $M$ (down to a maximum discount limit, e.g., -25%).
* The **Sponsored Party** wants to receive above $M$ (up to a maximum premium limit, e.g., +25%).
* Information asymmetry exists: neither party knows the other's exact walkaway/reservation limit.
* Outcomes incorporate dynamic parameters such as leverage, patience, concession decay, and random luck factors to ensure natural variance rather than a simple 50/50 midpoint split.

---

## 1. System Variables & State Setup

| Variable | Symbol | Description |
| :--- | :---: | :--- |
| **Median Benchmark** | $M$ | The reference market value (e.g., $100,000). |
| **Sponsor Walkaway Limit** | $RP_s$ | Max amount the Sponsor will pay: $M \times (1 + \text{Cap}_s)$, where $\text{Cap}_s \in [0, 0.25]$. |
| **Sponsored Walkaway Limit** | $RP_p$ | Min amount the Sponsored party will accept: $M \times (1 - \text{Floor}_p)$, where $\text{Floor}_p \in [0, 0.25]$. |
| **Opening Anchor (Sponsor)** | $A_s$ | Initial offer made by Sponsor (typically $M \times (1 - \text{Anchor}_s)$). |
| **Opening Anchor (Sponsored)** | $A_p$ | Initial offer made by Sponsored party (typically $M \times (1 + \text{Anchor}_p)$). |
| **Leverage Ratio** | $\alpha$ | Power distribution $\in [0, 1]$. Values $>0.5$ favor the Sponsored party; $<0.5$ favor the Sponsor. |
| **Concession Rate** | $k$ | Exponential decay factor determining how quickly concessions shrink over time. |
| **Patience / Max Rounds** | $T_{max}$ | Max rounds before stalemate occurs. |
| **Luck / Random Noise** | $\epsilon$ | Stochastic variable sampled from a Normal distribution $\mathcal{N}(0, \sigma^2)$. |

---

## 2. Core Logic Workflow

```
[Round t Begins]
       │
       ▼
Calculate Offers (Sponsor S_t, Sponsored P_t)
       │
       ▼
Check ZOPA Overlap (Is S_t >= P_t ?) ─── YES ───► [DEAL REACHED]
       │
       NO
       ▼
Check Walkaway / Timeout (t == T_max or Exceeded Limits)
       │
      YES ───► [NO DEAL / DEADLOCK]
       │
       NO
       ▼
Apply Concession Logic + Luck Factor + Stubbornness Decay ──► Proceed to Round t+1
```

---

## 3. Human Behavior Mathematical Models

### A. Tit-for-Tat Concession Decay (Stubbornness Curve)
Humans yield less as negotiations progress and as they draw closer to their walkaway limit.

* **Sponsor Offer Update:**
  $$S_{t+1} = S_t + \left( \Delta_{s, base} \times e^{-k_s \cdot t} \right) + \epsilon_t$$

* **Sponsored Offer Update:**
  $$P_{t+1} = P_t - \left( \Delta_{p, base} \times e^{-k_p \cdot t} \right) + \epsilon_t$$

Where $\Delta_{base}$ represents the base concession step and $k$ controls how quickly the party becomes rigid.

### B. Settlement Price Calculation (ZOPA Agreement)
When $S_t \ge P_t$, an agreement area (ZOPA) opens. The final settlement price is determined by relative leverage ($\alpha$) and remaining patience:

$$\text{Final Price} = P_t + \alpha \times (S_t - P_t)$$

* If Sponsor has high leverage ($\alpha = 0.2$), final price settles closer to $P_t$.
* If Sponsored party has high leverage ($\alpha = 0.8$), final price settles closer to $S_t$.

### C. The Luck Factor ($\epsilon$)
A random Gaussian noise term $\epsilon \sim \mathcal{N}(0, \sigma^2)$ is added to each turn to simulate real-world variance:
* **Positive Luck ($+\epsilon$):** Strong interpersonal chemistry, good market news, or sudden deadline pressure leading to a larger concession.
* **Negative Luck ($-\epsilon$):** Miscommunication, ego resistance, or sudden mistrust causing a party to hold firm or make minimal concessions.

---

## 4. Termination & Edge Cases

1. **Successful Agreement:** $S_t \ge P_t$ reached within $T_{max}$ rounds.
2. **Hard Deadlock:** $t = T_{max}$ reached without offer crossover.
3. **Walkaway Trigger:** If required concession forces an offer beyond $RP_s$ or $RP_p$, the party breaks off negotiations.