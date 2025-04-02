import type { RouteLocationRaw } from 'vue-router';

export interface IActionMenuItem {
  label: string;
  icon?: string;
  action?: () => Promise<unknown>;
  to?: RouteLocationRaw;
}
