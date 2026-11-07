"""Independent exact-rational P1.3 oracle; never imports Rust or writes files.

Smoothing uses the geometric closed form rather than the runtime recurrence.
Null denotes warmup, a string denotes undefined division, numbers denote ready.
"""
from fractions import Fraction as F


def evaluate(case):
    x, high, low, volume = ([F(str(v)) for v in case[key]]
                            for key in ["close", "high", "low", "volume"])
    n = case["period"]
    ranges = [max(high[i] - low[i], abs(high[i] - x[i-1]), abs(low[i] - x[i-1]))
              if i else high[0] - low[0] for i in range(len(x))]
    names = ["sma", "ema", "wma", "tr", "atr", "momentum", "roc", "volume_sma", "rvol"]
    result = {key: [] for key in names}

    def smoothing(values, index, alpha):
        k = index - (n - 1)
        seed = sum(values[:n]) / n
        return seed * (1-alpha)**k + alpha * sum(
            (1-alpha)**(k-j) * values[n-1+j] for j in range(1, k+1))

    for i in range(len(x)):
        row = dict.fromkeys(names)
        row["tr"] = ranges[i]
        if i >= n - 1:
            window = x[i+1-n:i+1]
            row.update(sma=sum(window)/n,
                       ema=smoothing(x, i, F(2, n+1)),
                       wma=sum((j+1)*v for j, v in enumerate(window))/F(n*(n+1), 2),
                       atr=smoothing(ranges, i, F(1, n)),
                       volume_sma=sum(volume[i+1-n:i+1])/n)
            row["rvol"] = volume[i]/row["volume_sma"] if row["volume_sma"] else "zero_denominator"
        if i >= n:
            row["momentum"] = x[i]-x[i-n]
            row["roc"] = 100*(x[i]/x[i-n]-1) if x[i-n] else "zero_denominator"
        for key, value in row.items():
            result[key].append(float(value) if isinstance(value, F) else value)
    return {"id": case["id"], **result}
