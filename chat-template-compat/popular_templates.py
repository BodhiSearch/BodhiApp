import yaml
import requests
from pathlib import Path
from gguf import GGUFReader
import asyncio
import aiohttp
from typing import Optional, Dict, Any
import logging

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class ModelProcessor:
    def __init__(self):
        self.base_dir = Path(__file__).parent
        self.repos_file = self.base_dir / "tests" / "data" / "embedded-repos.txt"
        self.output_file = self.base_dir / "tests" / "data" / "embedded-repos-with-base.yaml"

    async def get_base_model(self, session: aiohttp.ClientSession, repo: str) -> Optional[str]:
        url = f"https://huggingface.co/api/models/{repo}"
        try:
            async with session.get(url) as response:
                if response.status == 200:
                    data = await response.json()
                    return data.get("cardData", {}).get("base_model")
                else:
                    logger.error(f"Failed to fetch base model for {repo}: {response.status}")
                    return None
        except Exception as e:
            logger.error(f"Error fetching base model for {repo}: {e}")
            return None

    async def get_gguf_file_path(self, session: aiohttp.ClientSession, repo: str) -> Optional[str]:
        url = f"https://huggingface.co/api/models/{repo}/tree/main"
        try:
            async with session.get(url) as response:
                if response.status == 200:
                    files = await response.json()
                    gguf_file = next((f["path"] for f in files if f["path"].endswith(".gguf")), None)
                    return gguf_file
                else:
                    logger.error(f"Failed to fetch file list for {repo}: {response.status}")
                    return None
        except Exception as e:
            logger.error(f"Error fetching GGUF file path for {repo}: {e}")
            return None

    async def get_chat_template(self, repo: str, gguf_path: str) -> Optional[str]:
        url = f"https://huggingface.co/{repo}/resolve/main/{gguf_path}"
        try:
            reader = GGUFReader(url)
            metadata = reader.metadata
            return metadata.get("tokenizer.chat_template")
        except Exception as e:
            logger.error(f"Error getting chat template for {repo}: {e}")
            return None

    async def process_repo(self, session: aiohttp.ClientSession, repo: str) -> Optional[Dict[str, Any]]:
        logger.info(f"Processing repo: {repo}")
        
        base_model = await self.get_base_model(session, repo)
        if not base_model:
            return None

        gguf_path = await self.get_gguf_file_path(session, repo)
        if not gguf_path:
            return None

        chat_template = await self.get_chat_template(repo, gguf_path)
        if not chat_template:
            return None

        return {
            "id": repo,
            "base": base_model,
            "template": chat_template
        }

    async def process_all_repos(self):
        # Read repos file
        repos = self.repos_file.read_text().strip().split('\n')
        
        async with aiohttp.ClientSession() as session:
            tasks = [self.process_repo(session, repo) for repo in repos]
            results = await asyncio.gather(*tasks)
            
            # Filter out None results and create final data
            data = [result for result in results if result]
            
            # Write YAML file
            self.output_file.parent.mkdir(parents=True, exist_ok=True)
            with self.output_file.open('w') as f:
                yaml.dump(data, f, sort_keys=False, allow_unicode=True)
            
            logger.info(f"Processed {len(data)} repos successfully")

def main():
    processor = ModelProcessor()
    asyncio.run(processor.process_all_repos())

if __name__ == "__main__":
    main() 