export interface IHaexHubExtensionManifest {
  name: string;
  entry: string;
  permissions: {
    database?: {
      read?: string[];
      write?: string[];
      create?: string[];
    };
    http?: string[];
    filesystem?: {
      read?: string[];
      write?: string[];
    };
  };
}
