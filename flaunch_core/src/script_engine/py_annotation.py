flaunch_callables = {}

def flaunch(*args, **kwargs):
    def inner(func):
        flaunch_callables[func] = kwargs
    return inner
