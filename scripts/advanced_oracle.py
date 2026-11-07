"""Independent P1.4 oracle: exact fractions and closed-form weighted sums.

No Rust import or file writes. Smoothing is evaluated afresh at each index,
not with the runtime recurrence. Decimal square roots use 80-digit precision.
"""
from decimal import Decimal, localcontext
from fractions import Fraction as F


def smooth(values, n, alpha):
    seed = sum(values[:n]) / n
    return [None if i < n-1 else
            seed * (1-alpha)**(i-n+1) + alpha * sum(
                (1-alpha)**(i-j) * values[j] for j in range(n, i+1))
            for i in range(len(values))]


def evaluate(case):
    x, high, low = ([F(str(v)) for v in case[key]] for key in ["close", "high", "low"])
    n, f, s, q = (case[k] for k in ["period", "fast", "slow", "signal"])
    k = F(str(case["multiplier"]))
    names = ["rsi", "macd_line", "macd_signal", "macd_histogram", "plus_di",
             "minus_di", "dx", "adx", "bollinger_middle", "bollinger_upper", "bollinger_lower"]
    out = {key: [None] * len(x) for key in names}
    gains = [max(b-a, 0) for a, b in zip(x, x[1:])]
    losses = [max(a-b, 0) for a, b in zip(x, x[1:])]
    ag, al = smooth(gains, n, F(1, n)), smooth(losses, n, F(1, n))
    for i in range(n, len(x)):
        g, loss = ag[i-1], al[i-1]
        # Algebraically equivalent independent RSI expression.
        out["rsi"][i] = 100*g/(g+loss) if g+loss else F(50)
    fast, slow = smooth(x, f, F(2, f+1)), smooth(x, s, F(2, s+1))
    lines = [fast[i]-slow[i] for i in range(s-1, len(x))]
    signals = smooth(lines, q, F(2, q+1))
    for j, line in enumerate(lines):
        i = s-1+j
        out["macd_line"][i] = line
        if signals[j] is not None:
            out["macd_signal"][i] = signals[j]
            out["macd_histogram"][i] = line-signals[j]
    tr, plus, minus = [], [], []
    for i in range(1, len(x)):
        up, down = high[i]-high[i-1], low[i-1]-low[i]
        plus.append(up if up > max(down, 0) else F(0))
        minus.append(down if down > max(up, 0) else F(0))
        tr.append(max(high[i]-low[i], abs(high[i]-x[i-1]), abs(low[i]-x[i-1])))
    tr, plus, minus = (smooth(v, n, F(1, n)) for v in [tr, plus, minus])
    dxs = []
    for i in range(n, len(x)):
        p, m = (100*v[i-1]/tr[i-1] if tr[i-1] else F(0) for v in [plus, minus])
        dx = 100*abs(p-m)/(p+m) if p+m else F(0)
        out["plus_di"][i], out["minus_di"][i], out["dx"][i] = p, m, dx
        dxs.append(dx)
    adx = smooth(dxs, n, F(1, n))
    for j, v in enumerate(adx):
        out["adx"][n+j] = v
    for i in range(n-1, len(x)):
        window = x[i+1-n:i+1]
        mean = sum(window)/n
        # Pairwise distance identity, independent of runtime centered squares.
        variance = sum((a-b)**2 for a in window for b in window)/(2*n*n)
        with localcontext() as ctx:
            ctx.prec = 80
            dmean = Decimal(mean.numerator)/Decimal(mean.denominator)
            std = (Decimal(variance.numerator)/Decimal(variance.denominator)).sqrt()
            width = Decimal(k.numerator)/Decimal(k.denominator)*std
            out["bollinger_middle"][i] = mean
            out["bollinger_upper"][i] = dmean+width
            out["bollinger_lower"][i] = dmean-width
    return {"id": case["id"], **{key: [float(v) if v is not None else None for v in vs]
                                   for key, vs in out.items()}}
