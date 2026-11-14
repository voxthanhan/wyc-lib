"""Copy host values to wire DTOs; no analytical rules live here."""
import json
import math
import sys

_COLUMNS = ("timestamp", "available_at", "open", "high", "low", "close", "volume", "is_closed")
_TIME = {"as_of", "timestamp", "available_at", "revision_available_at", "after_timestamp",
         "before_timestamp", "last_timestamp", "last_as_of"}
_INTEGER = _TIME | {"period", "macd_fast", "macd_slow", "macd_signal", "left", "right"}
_SAFE = 9007199254740991


def fail(cls, code, path):
    error = cls(f"{code} at {path}")
    error.code, error.path = code, path
    raise error


def snapshot(value, root):
    if type(value) is not dict:
        fail(TypeError, "invalid_type", root)
    active = set()

    def copy(v, path, field="", depth=0):
        if depth > 64:
            fail(TypeError, "invalid_type", path)
        kind = type(v)
        if kind is float and not math.isfinite(v):
            fail(ValueError, "non_finite", path)
        if field in _INTEGER and v is not None and kind is not int and kind not in (list, tuple):
            # Arrays are checked below; float timestamps never silently truncate.
            np = sys.modules.get("numpy")
            if np is None or kind is not np.ndarray:
                fail(TypeError, "invalid_type", path)
        if kind is str:
            try:
                v.encode("utf-8")
            except UnicodeEncodeError:
                fail(TypeError, "invalid_type", path)
            return v
        if v is None or kind is bool:
            return v
        if kind is int:
            if abs(v) > _SAFE:
                fail(ValueError, "invalid_timestamp" if field in _TIME else "invalid_type", path)
            return v
        if kind is float:
            return v
        np = sys.modules.get("numpy")
        if np is not None and kind is np.ndarray:
            expected = {"timestamp": {("i", 8)}, "available_at": {("i", 8)},
                        "is_closed": {("b", 1)}}.get(field, {("f", 8), ("i", 8)})
            if field not in _COLUMNS or v.ndim != 1 or (v.dtype.kind, v.dtype.itemsize) not in expected:
                fail(TypeError, "invalid_type", path)
            v = v.tolist()
            kind = list
        if kind not in (dict, list, tuple) or id(v) in active:
            fail(TypeError, "invalid_type", path)
        active.add(id(v))
        try:
            if kind is dict:
                out = {}
                for key, item in v.items():
                    if type(key) is not str:
                        fail(TypeError, "invalid_type", path)
                    try:
                        key.encode("utf-8")
                    except UnicodeEncodeError:
                        fail(TypeError, "invalid_type", path)
                    out[key] = copy(item, f"{path}.{key}", key, depth + 1)
                return out
            return [copy(item, f"{path}[{i}]", field, depth + 1) for i, item in enumerate(v)]
        finally:
            active.remove(id(v))

    return copy(value, root)


def encode(value, root):
    return json.dumps(snapshot(value, root), ensure_ascii=False, allow_nan=False, separators=(",", ":"))


def _frame(frame, backend, symbol, timeframe, metadata, schema_version):
    if backend == "pandas":
        import pandas as pd
        if type(frame) is not pd.DataFrame:
            fail(TypeError, "invalid_type", "frame")
    else:
        import polars as pl
        if type(frame) is not pl.DataFrame:
            fail(TypeError, "invalid_type", "frame")
    columns = list(frame.columns)
    if (len(columns) != len(_COLUMNS) or any(type(c) is not str for c in columns)
            or set(columns) != set(_COLUMNS)):
        fail(TypeError, "invalid_type", "frame.columns")
    data = dict(schema_version=schema_version, symbol=symbol, timeframe=timeframe, metadata=metadata)
    for column in _COLUMNS:
        series = frame[column]
        if backend == "pandas":
            import numpy as np
            if not isinstance(series.dtype, np.dtype):
                fail(TypeError, "invalid_type", f"data.{column}")
            # Per-column arrays preserve integer precision and boolean types.
            data[column] = series.to_numpy(copy=True)
        else:
            expected = {"timestamp": (pl.Int64,), "available_at": (pl.Int64,),
                        "is_closed": (pl.Boolean,)}.get(column, (pl.Float64, pl.Int64))
            if series.dtype not in expected:
                fail(TypeError, "invalid_type", f"data.{column}")
            data[column] = series.to_list()
    return snapshot(data, "data")


def market_data_from_pandas(frame, *, symbol, timeframe, metadata, schema_version="0.1.0"):
    """Copy the eight explicit columns; the index is never interpreted as time."""
    return _frame(frame, "pandas", symbol, timeframe, metadata, schema_version)


def market_data_from_polars(frame, *, symbol, timeframe, metadata, schema_version="0.1.0"):
    """Copy the eight explicit columns; null remains None for Core validation."""
    return _frame(frame, "polars", symbol, timeframe, metadata, schema_version)
