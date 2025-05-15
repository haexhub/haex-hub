import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { join, resourceDir } from "@tauri-apps/api/path";
import { exists, readDir, readTextFile, remove } from "@tauri-apps/plugin-fs";
import { and, eq } from "drizzle-orm";
import type {
  IHaexHubExtension,
  IHaexHubExtensionLink,
  IHaexHubExtensionManifest,
} from "~/types/haexhub";
import { haexExtensions } from "~~/src-tauri/database/schemas/vault";

const manifestFileName = "manifest.json";
const logoFileName = "logo.svg";

export const useExtensionsStore = defineStore("extensionsStore", () => {
  const availableExtensions = ref<IHaexHubExtensionLink[]>([]);

  const extensionLinks = computed<ISidebarItem[]>(() =>
    availableExtensions.value
      .filter((extension) => extension.enabled && extension.installed)
      .map((extension) => ({
        icon: extension.icon ?? "",
        id: extension.id,
        name: extension.name ?? "",
        tooltip: extension.name ?? "",
        to: { name: "haexExtension", params: { extensionId: extension.id } },
      }))
  );

  const currentRoute = useRouter().currentRoute;

  const isActive = (id: string) =>
    computed(
      () => currentRoute.value.name === "extension" && currentRoute.value.params.extensionId === id
    );

  const currentExtension = computed(() => {
    console.log("computed currentExtension", currentRoute.value.params);
    if (currentRoute.value.meta.name !== "haexExtension") return;

    const extensionId = getSingleRouteParam(currentRoute.value.params.extensionId);
    console.log("extensionId from param", extensionId);
    if (!extensionId) return;

    const extension = availableExtensions.value.find((extension) => extension.id === extensionId);
    console.log("currentExtension", extension);
    return extension;
  });

  const getExtensionPathAsync = async (extensionId?: string, version?: string) => {
    if (!extensionId || !version) return "";
    return await join(await resourceDir(), "extensions", extensionId, version);
  };

  const checkSourceExtensionDirectoryAsync = async (extensionDirectory: string) => {
    try {
      const dir = await readDir(extensionDirectory);
      const manifest = dir.find((entry) => entry.name === manifestFileName && entry.isFile);
      if (!manifest) throw new Error("Kein Manifest für Erweiterung gefunden");

      const logo = dir.find((item) => item.isFile && item.name === logoFileName);
      if (!logo) throw new Error("Logo fehlt");

      return true;
    } catch (error) {
      throw new Error(`Keine Leseberechtigung für Ordner ${extensionDirectory}`);
    }
  };

  const isExtensionInstalledAsync = async (extension: Partial<IHaexHubExtension>) => {
    try {
      const extensionPath = await getExtensionPathAsync(extension.id, `${extension.version}`);
      console.log(`extension ${extension.id} is installed ${await exists(extensionPath)}`);
      return await exists(extensionPath);
    } catch (error) {
      return false;
    }
  };

  const checkManifest = (manifestFile: unknown): manifestFile is IHaexHubExtensionManifest => {
    const errors = [];

    if (typeof manifestFile !== "object" || manifestFile === null) {
      errors.push("Manifest ist falsch");
      return false;
    }

    if (!("id" in manifestFile) || typeof manifestFile.id !== "string")
      errors.push("Keine ID vergeben");

    if (!("name" in manifestFile) || typeof manifestFile.name !== "string")
      errors.push("Name fehlt");

    if (!("entry" in manifestFile) || typeof manifestFile.entry !== "string")
      errors.push("Entry fehlerhaft");

    if (!("author" in manifestFile) || typeof manifestFile.author !== "string")
      errors.push("Author fehlt");

    if (!("url" in manifestFile) || typeof manifestFile.url !== "string") errors.push("Url fehlt");

    if (!("version" in manifestFile) || typeof manifestFile.version !== "string")
      errors.push("Version fehlt");

    if (
      !("permissions" in manifestFile) ||
      typeof manifestFile.permissions !== "object" ||
      manifestFile.permissions === null
    ) {
      errors.push("Berechtigungen fehlen");
    }

    if (errors.length) throw errors;

    /* const permissions = manifestFile.permissions as Partial<IHaexHubExtensionManifest["permissions"]>;
    if (
      ("database" in permissions &&
        (typeof permissions.database !== "object" || permissions.database === null)) ||
      ("filesystem" in permissions && typeof permissions.filesystem !== "object") ||
      permissions.filesystem === null
    ) {
      return false;
    } */

    return true;
  };

  const readManifestFileAsync = async (extensionId: string, version: string) => {
    try {
      if (!(await isExtensionInstalledAsync({ id: extensionId, version }))) return null;

      const extensionPath = await getExtensionPathAsync(extensionId, `${version}`);
      const manifestPath = await join(extensionPath, manifestFileName);
      const manifest = (await JSON.parse(
        await readTextFile(manifestPath)
      )) as IHaexHubExtensionManifest;

      /*
      TODO implement check, that manifest has valid data
      */
      return manifest;
    } catch (error) {
      console.error("ERROR readManifestFileAsync", error);
    }
  };

  const installAsync = async (extensionDirectory: string | null, global: boolean = true) => {
    try {
      if (!extensionDirectory) throw new Error("Kein Ordner für Erweiterung angegeben");
      const manifestPath = await join(extensionDirectory, manifestFileName);
      const manifest = (await JSON.parse(
        await readTextFile(manifestPath)
      )) as IHaexHubExtensionManifest;

      const destination = await getExtensionPathAsync(manifest.id, manifest.version);

      await checkSourceExtensionDirectoryAsync(extensionDirectory);

      await invoke("copy_directory", { source: extensionDirectory, destination });

      const logoFilePath = await join(destination, "logo.svg");
      const logoSvg = await readTextFile(logoFilePath);

      const { currentVault } = storeToRefs(useVaultStore());
      const res = await currentVault.value?.drizzle.insert(haexExtensions).values({
        id: manifest.id,
        name: manifest.name,
        author: manifest.author,
        enabled: true,
        url: manifest.url,
        version: manifest.version,
        icon: logoSvg,
      });

      console.log("insert extensions", res);
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

  const extensionEntry = computedAsync(
    async () => {
      try {
        console.log("extensionEntry start", currentExtension.value);
        const regex = /((href|src)=["'])([^"']+)(["'])/g;

        if (!currentExtension.value?.id || !currentExtension.value.version) {
          console.log("extension id or entry missing", currentExtension.value);
          return "no mani: " + currentExtension.value;
        }

        const extensionPath = await getExtensionPathAsync(
          currentExtension.value?.id,
          currentExtension.value?.version
        ); //await join(await resourceDir(), currentExtension.value.. extensionDir, entryFileName);

        console.log("extensionEntry extensionPath", extensionPath);
        const manifest = await readManifestFileAsync(
          currentExtension.value.id,
          currentExtension.value.version
        );

        if (!manifest) return "no manifest readable";

        const entryPath = await join(extensionPath, manifest.entry);

        const hexName = stringToHex(
          JSON.stringify({
            id: currentExtension.value.id,
            version: currentExtension.value.version,
          })
        );

        return `haex-extension://${hexName}/index.html`;
        return convertFileSrc(entryPath); //`asset://localhost/${entryPath}`;
        let entryHtml = await readTextFile(entryPath);

        console.log("entryHtml", entryHtml);
        const replacements = [];
        let match;
        while ((match = regex.exec(entryHtml)) !== null) {
          const [fullMatch, prefix, attr, resource, suffix] = match;
          if (!resource.startsWith("http")) {
            replacements.push({ match: fullMatch, resource, prefix, suffix });
          }
        }

        for (const { match, resource, prefix, suffix } of replacements) {
          const srcFile = convertFileSrc(await join(extensionPath, resource));
          entryHtml = entryHtml.replace(match, `${prefix}${srcFile}${suffix}`);
        }

        console.log("entryHtml", entryHtml);

        const blob = new Blob([entryHtml], { type: "text/html" });
        const iframeSrc = URL.createObjectURL(blob);

        console.log("iframeSrc", iframeSrc);

        /* const path = convertFileSrc(extensionDir, manifest.entry);
    console.log("final path", path); */
        //manifest.entry = iframeSrc;
        return iframeSrc;
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
      } catch (error) {
        console.error("ERROR extensionEntry", error);
      }
    },
    null,
    { lazy: true }
  );

  const loadExtensionsAsync = async () => {
    const { currentVault } = storeToRefs(useVaultStore());

    const extensions = (await currentVault.value?.drizzle.select().from(haexExtensions)) ?? [];

    //if (!extensions?.length) return false;

    const installedExtensions = await filterAsync(extensions, isExtensionInstalledAsync);
    console.log("loadExtensionsAsync installedExtensions", installedExtensions);

    availableExtensions.value =
      extensions.map((extension) => ({
        id: extension.id,
        name: extension.name ?? "",
        icon: extension.icon ?? "",
        author: extension.author ?? "",
        version: extension.version ?? "",
        enabled: extension.enabled ? true : false,
        installed: installedExtensions.includes(extension),
      })) ?? [];

    console.log("loadExtensionsAsync", availableExtensions.value);
    return true;
  };

  const removeExtensionAsync = async (id: string, version: string) => {
    try {
      console.log("remove extension", id, version);
      await removeExtensionFromVaultAsync(id, version);
      await removeExtensionFilesAsync(id, version);
    } catch (error) {
      throw new Error(JSON.stringify(error));
    }
  };

  return {
    availableExtensions,
    checkManifest,
    currentExtension,
    extensionEntry,
    extensionLinks,
    installAsync,
    isActive,
    loadExtensionsAsync,
    readManifestFileAsync,
    removeExtensionAsync,
    getExtensionPathAsync,
  };
});

const getMimeType = (file: string) => {
  if (file.endsWith(".css")) return "text/css";
  if (file.endsWith(".js")) return "text/javascript";
  return "text/plain";
};

const removeExtensionFromVaultAsync = async (id: string | null, version: string | null) => {
  if (!id) throw new Error("Erweiterung kann nicht gelöscht werden. Es keine ID angegeben");

  if (!version)
    throw new Error("Erweiterung kann nicht gelöscht werden. Es wurde keine Version angegeben");

  const { currentVault } = useVaultStore();
  const removedExtensions = await currentVault?.drizzle
    .delete(haexExtensions)
    .where(and(eq(haexExtensions.id, id), eq(haexExtensions.version, version)));
  return removedExtensions;
};

const removeExtensionFilesAsync = async (id: string | null, version: string | null) => {
  try {
    const { getExtensionPathAsync } = useExtensionsStore();
    if (!id) throw new Error("Erweiterung kann nicht gelöscht werden. Es keine ID angegeben");

    if (!version)
      throw new Error("Erweiterung kann nicht gelöscht werden. Es wurde keine Version angegeben");

    const extensionDirectory = await getExtensionPathAsync(id, version);
    await remove(extensionDirectory, {
      recursive: true,
    });
  } catch (error) {
    console.error("ERROR removeExtensionFilesAsync", error);
    throw new Error(JSON.stringify(error));
  }
};

const replaceUrlWithAssetProtocolAsync = () => { };
