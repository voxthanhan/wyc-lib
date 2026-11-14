"""P1 owned Python API and backward-compatible JSON pilot; Rust owns analysis."""
import json
from . import _native
from ._conversion import encode as _encode, fail as _fail
from ._conversion import market_data_from_pandas, market_data_from_polars


class WycStateError(RuntimeError):
    pass


class WycEngineError(RuntimeError):
    pass


def _input(value, path):
    if not isinstance(value, str):
        error = TypeError(f"invalid_type at {path}")
        error.code, error.path = "invalid_type", path
        raise error
    return value


def _value(response):
    data = json.loads(response)
    if not data["ok"]:
        item = data["error"]
        cls = {"type": TypeError, "range": ValueError,
               "state": WycStateError, "engine": WycEngineError}[item["kind"]]
        error = cls(f'{item["code"]} at {item["path"]}')
        error.code, error.path = item["code"], item["path"]
        raise error
    return data["value"]


def _unwrap(response):
    return json.dumps(_value(response), ensure_ascii=False, allow_nan=False, separators=(",", ":"))


def analyze_batch(request):
    return _value(_native.analyze_json(_encode(request, "request")))


def _single(call, data, config, as_of):
    request = dict(schema_version="0.1.0", data=data, config=config, as_of=as_of, modules=[])
    return _value(call(_encode(request, "request")))


def calculate_indicators(data, config, as_of):
    return _single(_native.indicators_json, data, config, as_of)


def analyze_structure(data, config, as_of):
    return _single(_native.structure_json, data, config, as_of)


class AnalysisState:
    def __init__(self, request):
        self.__native = _native.NativeState()
        self.__closed = False
        _value(self.__native.reset_json(_encode(request, "request")))

    def _live(self):
        if self.__closed:
            _fail(WycStateError, "state_closed", "state")

    def result(self):
        self._live()
        return _value(self.__native.result_json())

    def identity(self):
        self._live()
        return _value(self.__native.identity_json())

    def update_closed_bar(self, update):
        self._live()
        return _value(self.__native.update_json(_encode(update, "update")))

    def reset(self, request):
        self._live()
        return _value(self.__native.reset_json(_encode(request, "request")))

    def close(self):
        self.__native.close()
        self.__closed = True

    def __enter__(self):
        self._live()
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        self.close()


def analyze_json(request):
    return _unwrap(_native.analyze_json(_input(request, "request")))


class State:
    def __init__(self, request):
        self.__native = _native.NativeState()
        _unwrap(self.__native.reset_json(_input(request, "request")))

    def result_json(self):
        return _unwrap(self.__native.result_json())

    def identity_json(self):
        return _unwrap(self.__native.identity_json())

    def update_json(self, update):
        return _unwrap(self.__native.update_json(_input(update, "update")))

    def reset_json(self, request):
        return _unwrap(self.__native.reset_json(_input(request, "request")))

    def close(self):
        self.__native.close()
