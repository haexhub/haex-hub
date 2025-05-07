//import { useRouter } from "vue-router";

// FlyonUI
import "flyonui/flyonui";

export default defineNuxtPlugin(() => {
  const router = useRouter();
  router.afterEach(async () => {
    setTimeout(() => window.HSStaticMethods.autoInit());
  });
});

/* import 'flyonui/flyonui';
import { type IStaticMethods } from 'flyonui/flyonui';
declare global {
  interface Window {
    HSStaticMethods: IStaticMethods;
  }
}

export default defineNuxtPlugin((nuxtApp) => {
  nuxtApp.hook('page:finish', () => {
    window.HSStaticMethods.autoInit();
  });
}); */
