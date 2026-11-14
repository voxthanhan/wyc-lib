"""Experimental P2.1 JSON transport; no analysis logic in Python."""
import json
from . import _native


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


def _unwrap(response):
    data = json.loads(response)
    if not data["ok"]:
        item = data["error"]
        cls = {"type": TypeError, "range": ValueError,
               "state": WycStateError, "engine": WycEngineError}[item["kind"]]
        error = cls(f'{item["code"]} at {item["path"]}')
        error.code, error.path = item["code"], item["path"]
        raise error
    return json.dumps(data["value"], ensure_ascii=False, allow_nan=False, separators=(",", ":"))


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
