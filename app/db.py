import asyncpg


async def create_pool(url: str) -> asyncpg.Pool:
    return await asyncpg.create_pool(dsn=url)


async def close_pool(pool: asyncpg.Pool) -> None:
    await pool.close()
