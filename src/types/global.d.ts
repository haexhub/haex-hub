import type { IStaticMethods } from "flyonui/flyonui";

declare global {
  interface Window {
    // Optional third-party libraries

    // FlyonUI
    HSStaticMethods: IStaticMethods;
  }
}

export {};
