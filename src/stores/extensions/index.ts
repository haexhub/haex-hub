import { invoke } from "@tauri-apps/api/core";
import { join, resourceDir } from "@tauri-apps/api/path";
import { readTextFile, readDir } from "@tauri-apps/plugin-fs";
import { convertFileSrc } from "@tauri-apps/api/core";

const manifestFileName = "manifest.json";

export interface IHaexHubExtensionLink {
  name: string;
  icon: string;
  tooltip?: string;
  id: string;
}

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

export const useExtensionsStore = defineStore("extensionsStore", () => {
  const availableExtensions = ref<IHaexHubExtensionLink[]>([
    {
      id: "haex-browser",
      name: "Haex Browser",
      icon: "solar:global-outline",
    },

    {
      id: "extensions",
      name: "sidebar.extensions",
      icon: "gg:extension",
    },

    {
      id: "settings",
      name: "sidebar.settings",
      icon: "ph:gear-six",
    },
  ]);

  const currentRoute = useRouter().currentRoute;

  const isActive = (id: string) =>
    computed(
      () =>
        currentRoute.value.name === "extension" &&
        currentRoute.value.params.extensionId === id
    );

  const currentExtension = computed(() => {
    if (currentRoute.value.name !== "haexExtension") return;

    const extensionId = getSingleRouteParam(
      currentRoute.value.params.extensionId
    );

    if (!extensionId) return;

    return availableExtensions.value.find(
      (extension) => extension.id === extensionId
    );
  });

  const checkExtensionDirectoryAsync = async (extensionDirectory: string) => {
    try {
      const dir = await readDir(extensionDirectory);
      const manifest = dir.find(
        (entry) => entry.name === manifestFileName && entry.isFile
      );
      if (!manifest) throw new Error("Kein Manifest für Erweiterung gefunden");
      return true;
    } catch (error) {
      throw new Error(
        `Keine Leseberechtigung für Ordner ${extensionDirectory}`
      );
    }
  };

  const installAsync = async (
    extensionDirectory: string | null,
    global: boolean = true
  ): Promise<void> => {
    try {
      if (!extensionDirectory)
        throw new Error("Kein Ordner für Erweiterung angegeben");
      const checkDirectory = await checkExtensionDirectoryAsync(
        extensionDirectory
      );

      const manifestPath = await join(extensionDirectory, "manifest.json");
      const manifest = await JSON.parse(await readTextFile(manifestPath));

      console.log("manifest", manifest);
      return;
    } catch (error) {
      throw error;
      /*
      const resourcePath = await resourceDir();
      //const manifestPath = await join(extensionDirectory, 'manifest.json');
      const manifestPath = await join(
        resourcePath,
        'extension',
        'demo-addon',
        'manifest.json'
      );
      const regex = /((href|src)=["'])([^"']+)(["'])/g;
      let htmlContent = await readTextFile(
        await join(resourcePath, 'extension', 'demo-addon', 'index.html')
      );

      const replacements = [];
      let match;
      while ((match = regex.exec(htmlContent)) !== null) {
        const [fullMatch, prefix, attr, resource, suffix] = match;
        if (!resource.startsWith('http')) {
          replacements.push({ match: fullMatch, resource, prefix, suffix });
        }
      }

      for (const { match, resource, prefix, suffix } of replacements) {
             const fileContent = await readTextFile(
          await join(resourcePath, 'extension', 'demo-addon', resource)
        );
        const blob = new Blob([fileContent], { type: getMimeType(resource) });
        const blobUrl = URL.createObjectURL(blob);
        console.log('blob', resource, blobUrl);
        htmlContent = htmlContent.replace(
          match,
          `${prefix}${blobUrl}${suffix}`
        );
      }

      console.log('htmlContent', htmlContent);

      const blob = new Blob([htmlContent], { type: 'text/html' });
      const iframeSrc = URL.createObjectURL(blob);

      const manifestContent = await readTextFile(manifestPath);
      console.log('iframeSrc', iframeSrc);
      const manifest: PluginManifest = JSON.parse(manifestContent);
      //const entryPath = await join(extensionDirectory, manifest.entry);
      const entryPath = await join(
        resourcePath,
        'extension',
        'demo-addon',
        manifest.entry
      );
      console.log('extensionDirectory', extensionDirectory, entryPath);
      const path = convertFileSrc(extensionDirectory, manifest.entry);
      console.log('final path', path);
      manifest.entry = iframeSrc;
      /* await join(
        path, //`file:/${extensionDirectory}`,
        manifest.entry
      ); */
      // Modul-Datei laden
      //const modulePathFull = await join(basePath, manifest.main);
      /* const manifest: PluginManifest = await invoke('load_plugin', {
        manifestPath,
      }); */
      /* const iframe = document.createElement('iframe');
      iframe.src = manifest.entry;
      iframe.setAttribute('sandbox', 'allow-scripts');
      iframe.style.width = '100%';
      iframe.style.height = '100%';
      iframe.style.border = 'none'; */
      /* const addonApi = {
        db_execute: async (sql: string, params: string[] = []) => {
          return invoke('db_execute', {
            addonId: manifest.name,
            sql,
            params,
          });
        },
        db_select: async (sql: string, params: string[] = []) => {
          return invoke('db_select', {
            addonId: manifest.name,
            sql,
            params,
          });
        },
      }; */
      /* iframe.onload = () => {
        iframe.contentWindow?.postMessage(
          { type: 'init', payload: addonApi },
          '*'
        );
      };

      window.addEventListener('message', (event) => {
        if (event.source === iframe.contentWindow) {
          const { type } = event.data;
          if (type === 'ready') {
            console.log(`Plugin ${manifest.name} ist bereit`);
          }
        }
      }); */
      /* plugins.value.push({ name: manifest.name, entry: manifest.entry });

        console.log(`Plugin ${manifest.name} geladen.`); */
    }
  };

  return {
    availableExtensions,
    currentExtension,
    isActive,
  };
});
