from collections.abc import AsyncIterator
from contextlib import asynccontextmanager

from fastapi import FastAPI

from app.config import Config
from app.db import close_pool, create_pool


@asynccontextmanager
async def lifespan(app: FastAPI) -> AsyncIterator[None]:
    app.state.config = Config()
    app.state.pool = await create_pool(app.state.config.database_url)
    yield
    await close_pool(app.state.pool)
