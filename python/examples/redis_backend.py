import asyncio
import json
import redis.asyncio as redis
from tryx.backend import StoreBase

class RedisStore(StoreBase):
    def __init__(self, url="redis://localhost"):
        self.r = redis.from_url(url)
    
    async def put_identity(self, address: str, key: bytes) -> None:
        await self.r.set(f"identity:{address}", key)
        
    async def load_identity(self, address: str) -> bytes | None:
        return await self.r.get(f"identity:{address}")
        
    async def delete_identity(self, address: str) -> None:
        await self.r.delete(f"identity:{address}")

    async def get_session(self, address: str) -> bytes | None:
        return await self.r.get(f"session:{address}")

    # ... we will just implement what we need for testing
    
    def __getattr__(self, name):
        # for missing methods, mock them
        async def mock(*args, **kwargs):
            return None
        return mock

async def main():
    store = RedisStore()
    await store.put_identity("test_addr", b"test_key")
    val = await store.load_identity("test_addr")
    print("Loaded from redis:", val)

if __name__ == "__main__":
    asyncio.run(main())
