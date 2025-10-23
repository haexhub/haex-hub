CREATE TABLE `haex_crdt_configs` (
	`key` text PRIMARY KEY NOT NULL,
	`value` text
);
--> statement-breakpoint
CREATE TABLE `haex_crdt_logs` (
	`id` text PRIMARY KEY NOT NULL,
	`haex_timestamp` text,
	`table_name` text,
	`row_pks` text,
	`op_type` text,
	`column_name` text,
	`new_value` text,
	`old_value` text
);
--> statement-breakpoint
CREATE INDEX `idx_haex_timestamp` ON `haex_crdt_logs` (`haex_timestamp`);--> statement-breakpoint
CREATE INDEX `idx_table_row` ON `haex_crdt_logs` (`table_name`,`row_pks`);--> statement-breakpoint
CREATE TABLE `haex_crdt_snapshots` (
	`snapshot_id` text PRIMARY KEY NOT NULL,
	`created` text,
	`epoch_hlc` text,
	`location_url` text,
	`file_size_bytes` integer
);
--> statement-breakpoint
CREATE TABLE `haex_desktop_items` (
	`id` text PRIMARY KEY NOT NULL,
	`workspace_id` text NOT NULL,
	`item_type` text NOT NULL,
	`reference_id` text NOT NULL,
	`position_x` integer DEFAULT 0 NOT NULL,
	`position_y` integer DEFAULT 0 NOT NULL,
	`haex_timestamp` text,
	FOREIGN KEY (`workspace_id`) REFERENCES `haex_workspaces`(`id`) ON UPDATE no action ON DELETE cascade
);
--> statement-breakpoint
CREATE TABLE `haex_extension_permissions` (
	`id` text PRIMARY KEY NOT NULL,
	`extension_id` text NOT NULL,
	`resource_type` text,
	`action` text,
	`target` text,
	`constraints` text,
	`status` text DEFAULT 'denied' NOT NULL,
	`created_at` text DEFAULT (CURRENT_TIMESTAMP),
	`updated_at` integer,
	`haex_timestamp` text,
	FOREIGN KEY (`extension_id`) REFERENCES `haex_extensions`(`id`) ON UPDATE no action ON DELETE cascade
);
--> statement-breakpoint
CREATE UNIQUE INDEX `haex_extension_permissions_extension_id_resource_type_action_target_unique` ON `haex_extension_permissions` (`extension_id`,`resource_type`,`action`,`target`);--> statement-breakpoint
CREATE TABLE `haex_extensions` (
	`id` text PRIMARY KEY NOT NULL,
	`public_key` text NOT NULL,
	`name` text NOT NULL,
	`version` text NOT NULL,
	`author` text,
	`description` text,
	`entry` text DEFAULT 'index.html' NOT NULL,
	`homepage` text,
	`enabled` integer DEFAULT true,
	`icon` text,
	`signature` text NOT NULL,
	`haex_timestamp` text
);
--> statement-breakpoint
CREATE UNIQUE INDEX `haex_extensions_public_key_name_unique` ON `haex_extensions` (`public_key`,`name`);--> statement-breakpoint
CREATE TABLE `haex_notifications` (
	`id` text PRIMARY KEY NOT NULL,
	`alt` text,
	`date` text,
	`icon` text,
	`image` text,
	`read` integer,
	`source` text,
	`text` text,
	`title` text,
	`type` text NOT NULL,
	`haex_timestamp` text
);
--> statement-breakpoint
CREATE TABLE `haex_settings` (
	`id` text PRIMARY KEY NOT NULL,
	`key` text,
	`type` text,
	`value` text,
	`haex_timestamp` text
);
--> statement-breakpoint
CREATE UNIQUE INDEX `haex_settings_key_type_value_unique` ON `haex_settings` (`key`,`type`,`value`);--> statement-breakpoint
CREATE TABLE `haex_workspaces` (
	`id` text PRIMARY KEY NOT NULL,
	`name` text NOT NULL,
	`position` integer DEFAULT 0 NOT NULL,
	`haex_timestamp` text
);
--> statement-breakpoint
CREATE UNIQUE INDEX `haex_workspaces_position_unique` ON `haex_workspaces` (`position`);