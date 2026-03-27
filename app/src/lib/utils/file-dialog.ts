import { open } from '@tauri-apps/plugin-dialog';

interface FileFilter {
  name: string;
  extensions: string[];
}

export async function selectFile(filters: FileFilter[]): Promise<string | null> {
  const result = await open({ filters });
  return result ? (result as string) : null;
}

export async function selectDirectory(): Promise<string | null> {
  const result = await open({ directory: true });
  return result ? (result as string) : null;
}

export async function selectFiles(filters: FileFilter[]): Promise<string[] | null> {
  const result = await open({ filters, multiple: true });
  return result ? (result as string[]) : null;
}
