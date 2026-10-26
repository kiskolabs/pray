export interface ProjectInvocationContext {
  projectRoot: string;
  manifestPath: string;
  environment?: string;
}

export interface ProjectInvocationOptions {
  projectRoot?: string;
  manifestPath?: string;
  environment?: string;
}
