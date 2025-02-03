import fs from 'fs';
import path from 'path';

const DOCS_DIR_NAME = 'src/docs';
const MD_EXTENSION = '.md';

export function getAllDocPaths() {
  const docsDirectory = path.join(process.cwd(), DOCS_DIR_NAME);

  const getAllFiles = (
    dirPath: string,
    arrayOfFiles: string[] = []
  ): string[] => {
    try {
      const files = fs.readdirSync(dirPath);

      files.forEach((file) => {
        const filePath = path.join(dirPath, file);
        if (fs.statSync(filePath).isDirectory()) {
          arrayOfFiles = getAllFiles(filePath, arrayOfFiles);
        } else if (path.extname(file) === MD_EXTENSION) {
          const relativePath = path.relative(docsDirectory, filePath);
          arrayOfFiles.push(relativePath.replace(/\.md$/, ''));
        }
      });

      return arrayOfFiles;
    } catch (e) {
      console.error('Error reading docs directory:', e);
      return [];
    }
  };

  return getAllFiles(docsDirectory);
}
