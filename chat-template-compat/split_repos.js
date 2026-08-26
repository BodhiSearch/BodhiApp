import fs from 'fs/promises';
import path from 'path';
import yaml from 'js-yaml';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

async function main() {
  try {
    const baseYamlPath = path.join(__dirname, 'tests', 'data', 'embedded-repos-with-base.yaml');
    const varsYamlPath = path.join(__dirname, 'tests', 'data', 'embedded-repos-with-vars.yaml');
    const baseYaml = await fs.readFile(baseYamlPath, 'utf8');
    const varsYaml = await fs.readFile(varsYamlPath, 'utf8');

    const baseData = yaml.load(baseYaml);
    const varsData = yaml.load(varsYaml);

    const varsMap = new Map(varsData.map(item => [item.id, item]));

    const outputDir = path.join(__dirname, 'tests', 'data', 'test-inputs');
    await fs.mkdir(outputDir, { recursive: true });

    for (const baseItem of baseData) {
      let varsItem = varsMap.get(baseItem.id);
      if (!varsItem) {
        console.warn(`No vars data found for ${baseItem.id}`);
        varsItem = {
          id: baseItem.id,
          variables: []
        };
      }

      const mergedItem = {
        ...varsItem,
        ...baseItem,
      };

      const [author, ...repoParts] = baseItem.id.split('/');
      const repo = repoParts.join('--');
      const filename = `${author}--${repo}.yaml`;

      const outputPath = path.join(outputDir, filename);
      await fs.writeFile(
        outputPath,
        yaml.dump(mergedItem, { lineWidth: -1, noRefs: true })
      );

      console.log(`Created ${filename}`);
    }

    console.log('Done!');
  } catch (error) {
    console.error('Error:', error);
  }
}

main(); 