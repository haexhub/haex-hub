(function () {
  // Überwache das Erstellen von Elementen, die Ressourcen laden
  const originalCreateElement = document.createElement;
  document.createElement = function (tagName) {
    const element = originalCreateElement.call(document, tagName);

    if (
      tagName.toLowerCase() === 'img' ||
      tagName.toLowerCase() === 'script' ||
      tagName.toLowerCase() === 'iframe'
    ) {
      // Überwache das Setzen des src-Attributs
      const originalSetAttribute = element.setAttribute;
      element.setAttribute = function (name, value) {
        if (name === 'src') {
          // Prüfe, ob die Ressource blockiert werden soll
          window.__TAURI__
            .invoke('block_resource_request', {
              url: value,
              resourceType: tagName.toLowerCase(),
            })
            .then((shouldBlock) => {
              if (shouldBlock) {
                console.log(`Ressourcenanfrage blockiert: ${value}`);
                return;
              }
              originalSetAttribute.call(element, name, value);
            });
        } else {
          originalSetAttribute.call(element, name, value);
        }
      };
    }

    return element;
  };

  // Wenn die Tauri HTTP API verwendet wird, können wir sie hier überwachen
  if (window.__TAURI__ && window.__TAURI__.http) {
    const originalFetch = window.__TAURI__.http.fetch;
    window.__TAURI__.http.fetch = async function (options) {
      // Prüfe, ob die Ressource blockiert werden soll
      const shouldBlock = await invoke('block_resource_request', {
        url: options.url,
        resourceType: 'tauri-fetch',
      });

      if (shouldBlock) {
        throw new Error(`Ressourcenanfrage blockiert: ${options.url}`);
      }

      return originalFetch.call(this, options);
    };
  }

  console.log('Webview-Bridge geladen');
})();
