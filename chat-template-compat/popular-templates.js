import { gguf } from "@huggingface/gguf";
import fs from 'fs/promises';
import path from 'path';
import { fileURLToPath } from 'url';
import yaml from 'js-yaml';

// Get current script directory
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Compute output directory relative to script location
const OUTPUT_DIR = path.join(__dirname, 'tests', 'data', 'embedded');
const REPOS_FILE = path.join(__dirname, 'tests', 'data', 'embedded-repos.txt');
const OUTPUT_YAML = path.join(__dirname, 'tests', 'data', 'embedded-repos-with-base.yaml');

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

async function fetchBaseModel(repo) {
  const url = `https://huggingface.co/api/models/${repo}`;
  try {
    const response = await fetch(url);
    if (!response.ok) {
      console.error(`Failed to fetch base model for ${repo}: ${response.statusText}`);
      return null;
    }
    const data = await response.json();
    return data?.cardData?.base_model;
  } catch (error) {
    console.error(`Error fetching base model for ${repo}:`, error);
    return null;
  }
}

async function getModelInfo(repo) {
  try {
    console.log(`Processing ${repo}...`);
    
    const baseModel = await fetchBaseModel(repo);
    if (!baseModel) {
      console.log(`No base model found for ${repo}`);
      return null;
    }

    const filename = await fetchFirstGGUFFile(repo);
    const url = `https://huggingface.co/${repo}/resolve/main/${filename}`;
    console.log(`Reading GGUF file from ${url}...`);

    const { metadata } = await gguf(url);
    const template = metadata?.['tokenizer.chat_template'];
    
    if (!template) {
      console.log(`No chat template found for ${repo}/${filename}`);
      return null;
    }

    return {
      id: repo,
      base: baseModel,
      template: template
    };
  } catch (error) {
    console.error(`Error processing ${repo}:`, error);
    return null;
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

    // Process all repos in parallel with concurrency limit
    const concurrencyLimit = 5;
    const results = [];
    
    for (let i = 0; i < modelRepos.length; i += concurrencyLimit) {
      const batch = modelRepos.slice(i, i + concurrencyLimit);
      const batchResults = await Promise.all(batch.map(repo => getModelInfo(repo)));
      results.push(...batchResults.filter(Boolean));
    }

    // Save results to YAML file
    await ensureDirectoryExists(path.dirname(OUTPUT_YAML));
    await fs.writeFile(
      OUTPUT_YAML,
      yaml.dump(results, { lineWidth: -1, noRefs: true })
    );
    
    console.log(`Successfully processed ${results.length} models`);
    console.log(`Results saved to ${OUTPUT_YAML}`);
  } catch (error) {
    console.error('Error in main:', error);
  }
}

main(); 