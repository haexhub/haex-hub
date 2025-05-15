export interface IHaexHubExtensionManifest {
  name: string;
  id: string;
  entry: string;
  author: string;
  url: string;
  version: string;
  icon: string;
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

export interface IHaexHubExtensionLink extends IHaexHubExtension {
  installed: boolean;
}

export interface IHaexHubExtension {
  author?: string | null;
  enabled?: boolean | null;
  icon?: string | null;
  id: string;
  manifest?: IHaexHubExtensionManifest;
  name: string | null;
  version?: string | null;
}
