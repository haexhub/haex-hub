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
CREATE TABLE `haex_extension_permissions` (
	`id` text PRIMARY KEY NOT NULL,
	`extension_id` text,
	`resource_type` text,
	`action` text,
	`target` text,
	`constraints` text,
	`status` text DEFAULT 'denied' NOT NULL,
	`created_at` text DEFAULT (CURRENT_TIMESTAMP),
	`updated_at` integer,
	`haex_tombstone` integer,
	`haex_timestamp` text,
	FOREIGN KEY (`extension_id`) REFERENCES `haex_extensions`(`id`) ON UPDATE no action ON DELETE no action
);
--> statement-breakpoint
CREATE UNIQUE INDEX `haex_extension_permissions_extension_id_resource_type_action_target_unique` ON `haex_extension_permissions` (`extension_id`,`resource_type`,`action`,`target`);--> statement-breakpoint
CREATE TABLE `haex_extensions` (
	`id` text PRIMARY KEY NOT NULL,
	`author` text,
	`description` text,
	`entry` text,
	`homepage` text,
	`enabled` integer,
	`icon` text,
	`name` text,
	`public_key` text,
	`signature` text,
	`url` text,
	`version` text,
	`haex_tombstone` integer,
	`haex_timestamp` text
);
--> statement-breakpoint
CREATE TABLE `haex_settings` (
	`id` text PRIMARY KEY NOT NULL,
	`key` text,
	`type` text,
	`value` text,
	`haex_tombstone` integer,
	`haex_timestamp` text
);
--> statement-breakpoint
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
	`haex_tombstone` integer
);
--> statement-breakpoint
CREATE TABLE `haex_passwords_group_items` (
	`group_id` text,
	`item_id` text,
	`haex_tombstone` integer,
	PRIMARY KEY(`item_id`, `group_id`),
	FOREIGN KEY (`group_id`) REFERENCES `haex_passwords_groups`(`id`) ON UPDATE no action ON DELETE no action,
	FOREIGN KEY (`item_id`) REFERENCES `haex_passwords_item_details`(`id`) ON UPDATE no action ON DELETE no action
);
--> statement-breakpoint
CREATE TABLE `haex_passwords_groups` (
	`id` text PRIMARY KEY NOT NULL,
	`name` text,
	`description` text,
	`icon` text,
	`order` integer,
	`color` text,
	`parent_id` text,
	`created_at` text DEFAULT (CURRENT_TIMESTAMP),
	`updated_at` integer,
	`haex_tombstone` integer,
	FOREIGN KEY (`parent_id`) REFERENCES `haex_passwords_groups`(`id`) ON UPDATE no action ON DELETE no action
);
--> statement-breakpoint
CREATE TABLE `haex_passwords_item_details` (
	`id` text PRIMARY KEY NOT NULL,
	`title` text,
	`username` text,
	`password` text,
	`note` text,
	`icon` text,
	`tags` text,
	`url` text,
	`created_at` text DEFAULT (CURRENT_TIMESTAMP),
	`updated_at` integer,
	`haex_tombstone` integer
);
--> statement-breakpoint
CREATE TABLE `haex_passwords_item_history` (
	`id` text PRIMARY KEY NOT NULL,
	`item_id` text,
	`changed_property` text,
	`old_value` text,
	`new_value` text,
	`created_at` text DEFAULT (CURRENT_TIMESTAMP),
	`haex_tombstone` integer,
	FOREIGN KEY (`item_id`) REFERENCES `haex_passwords_item_details`(`id`) ON UPDATE no action ON DELETE no action
);
--> statement-breakpoint
CREATE TABLE `haex_passwords_item_key_values` (
	`id` text PRIMARY KEY NOT NULL,
	`item_id` text,
	`key` text,
	`value` text,
	`updated_at` integer,
	`haex_tombstone` integer,
	FOREIGN KEY (`item_id`) REFERENCES `haex_passwords_item_details`(`id`) ON UPDATE no action ON DELETE no action
);
