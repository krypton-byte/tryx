from ._tryx import client # type: ignore

for name in dir(client): # type: ignore
    obj = getattr(client, name) # type: ignore
    if isinstance(obj, type):
        globals()[name] = obj

__all__ = [name for name in dir(client) if isinstance(getattr(client, name), type)] # type: ignore