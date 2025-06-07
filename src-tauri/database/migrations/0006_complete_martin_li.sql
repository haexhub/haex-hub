ALTER TABLE `haex_extensions_permissions` ADD `created_at` text DEFAULT (CURRENT_TIMESTAMP);--> statement-breakpoint
ALTER TABLE `haex_extensions_permissions` ADD `updated_at` integer;--> statement-breakpoint
ALTER TABLE `haex_passwords_groups` ADD `created_at` text DEFAULT (CURRENT_TIMESTAMP);--> statement-breakpoint
ALTER TABLE `haex_passwords_groups` ADD `updated_at` integer;--> statement-breakpoint
ALTER TABLE `haex_passwords_items_key_values` ADD `updated_at` integer;