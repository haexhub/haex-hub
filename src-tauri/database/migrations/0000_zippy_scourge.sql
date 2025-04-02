CREATE TABLE `haex_extensions` (
	`id` text PRIMARY KEY NOT NULL,
	`author` text,
	`enabled` integer,
	`name` text,
	`url` text,
	`version` text
);
--> statement-breakpoint
CREATE TABLE `haex_extensions_permissions` (
	`id` text PRIMARY KEY NOT NULL,
	`extension_id` text,
	`resource` text,
	`operation` text,
	`path` text,
	FOREIGN KEY (`extension_id`) REFERENCES `haex_extensions`(`id`) ON UPDATE no action ON DELETE no action
);
--> statement-breakpoint
CREATE UNIQUE INDEX `haex_extensions_permissions_extension_id_resource_operation_path_unique` ON `haex_extensions_permissions` (`extension_id`,`resource`,`operation`,`path`);--> statement-breakpoint
CREATE TABLE `haex_settings` (
	`id` text PRIMARY KEY NOT NULL,
	`key` text,
	`value_text` text,
	`value_json` text,
	`value_number` numeric
);
