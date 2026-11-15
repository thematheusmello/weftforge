"""Current source identity; old two-case entry points are deliberately unavailable."""
SOURCE = "ad7cc8bbbfc7a1ae735a400c21dcc7a37c80a5c5"

def unavailable(*args, **kwargs):
    raise RuntimeError("Use this packet run.py with the separately issued root GO")

verify_preparation = verify_frozen = verify_readonly_inputs = authorization = unavailable
