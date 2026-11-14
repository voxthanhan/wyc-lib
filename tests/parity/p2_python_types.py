"""Strict installed-package consumer; unused ignores must fail mypy."""
from typing import TYPE_CHECKING
import wyc_lib as api
if TYPE_CHECKING:
    from wyc_lib import AnalysisRequest, NumericValue


def consume(request: 'AnalysisRequest') -> float:
    result = api.analyze_batch(request)
    assert result['metadata']['as_of'] == request['as_of']
    indicators = api.calculate_indicators(request['data'], request['config'], request['as_of'])
    sample: NumericValue = indicators['output']['series']['sma'][0]['result']
    value = 0.0
    if sample['status'] == 'ready':
        value = sample['value']
    elif sample['status'] == 'warmup':
        value = float(sample['required'] - sample['available'])
    else:
        assert sample['reason'] in ('zero_denominator', 'unavailable_dependency')
    with api.AnalysisState(request) as state:
        identity = state.identity()
        state.reset(request)
        assert identity['last_as_of'] == request['as_of']
    api.analyze_structure(request['data'], request['config'], request['as_of'])
    api.analyze_batch('bad')  # type: ignore[arg-type]
    wrong: NumericValue = {'status': 'ready', 'value': 'bad'}  # type: ignore[typeddict-item]
    indicators['output']['series']['unknown']  # type: ignore[index]
    api.calculate_indicators(request['data'], request['config'], 'bad')  # type: ignore[arg-type]
    assert wrong
    return value
