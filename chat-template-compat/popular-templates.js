import { gguf } from "@huggingface/gguf";
import fs from 'fs/promises';
import path from 'path';
import { fileURLToPath } from 'url';

// Get current script directory
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Compute output directory relative to script location
const OUTPUT_DIR = path.join(__dirname, 'tests', 'data', 'embedded');
const REPOS_FILE = path.join(__dirname, 'tests', 'data', 'embedded-repos.txt');

async function ensureDirectoryExists(dirPath) {
  try {
    await fs.mkdir(dirPath, { recursive: true });
  } catch (error) {
    if (error.code !== 'EEXIST') throw error;
  }
}

async function fileExists(filePath) {
  try {
    await fs.access(filePath);
    return true;
  } catch {
    return false;
  }
}

async function fetchOwnerModels(owner, limit = 500) {
  let allModels = [];
  const url = `https://huggingface.co/api/models?author=${owner}&limit=${limit}`;
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Failed to fetch models for ${owner}: ${response.statusText}`);
  }
  const models = await response.json();
  const filteredModels = models
    .filter(model => model.modelId.startsWith(owner));
  allModels.push(...filteredModels.map(model => model.modelId));
  return allModels;
}

async function fetchFirstGGUFFile(owner_repo) {
  const url = `https://huggingface.co/api/models/${owner_repo}/tree/main`;
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Failed to fetch file list for ${owner_repo}: ${response.statusText}`);
  }

  const files = await response.json();
  const ggufFile = files.find(file => file.path.endsWith('.gguf'));

  if (!ggufFile) {
    throw new Error(`No GGUF file found in ${owner_repo}`);
  }

  return ggufFile.path;
}

async function readAndSaveGGUF(owner_repo) {
  try {
    const sanitizedName = `${owner_repo.replace('/', '--')}`;
    const outputPath = path.join(OUTPUT_DIR, `${sanitizedName}.j2`);

    // Skip if file already exists
    if (await fileExists(outputPath)) {
      console.log(`Template already exists for ${owner_repo}, skipping...`);
      return;
    }

    const filename = await fetchFirstGGUFFile(owner_repo);
    const url = `https://huggingface.co/${owner_repo}/resolve/main/${filename}`;
    console.log(`Reading GGUF file from ${url}...`);

    const { metadata } = await gguf(url);

    if (metadata?.['tokenizer.chat_template']) {
      const template = metadata['tokenizer.chat_template'];

      await ensureDirectoryExists(OUTPUT_DIR);
      await fs.writeFile(outputPath, template);

      console.log(`Saved chat template for ${owner_repo}/${filename} to ${outputPath}`);
    } else {
      console.log(`No chat template found for ${owner_repo}/${filename}`);
    }
  } catch (error) {
    console.error(`Error processing ${owner_repo}:`, error);
  }
}

async function main() {
  const owner = 'lmstudio-community';
  try {
    const modelRepos = await fetchOwnerModels(owner);
    console.log(`Found ${modelRepos.length} model repos for ${owner}`);

    // Save repos list to file
    await ensureDirectoryExists(path.dirname(REPOS_FILE));
    await fs.writeFile(REPOS_FILE, modelRepos.join('\n'));
    console.log(`Saved repos list to ${REPOS_FILE}`);

    // for (const repo of modelRepos) {
    //   await readAndSaveGGUF(repo);
    // }
  } catch (error) {
    console.error('Error in main:', error);
  }
}

main(); 