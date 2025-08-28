CREATE TABLE `haex_crdt_logs` (
	`hlc_timestamp` text PRIMARY KEY NOT NULL,
	`table_name` text,
	`row_pks` text,
	`op_type` text,
	`column_name` text,
	`new_value` text,
	`old_value` text
);
--> statement-breakpoint
CREATE TABLE `haex_crdt_settings` (
	`type` text PRIMARY KEY NOT NULL,
	`value` text
);
--> statement-breakpoint
CREATE TABLE `haex_crdt_snapshots` (
	`snapshot_id` text PRIMARY KEY NOT NULL,
	`created` text,
	`epoch_hlc` text,
	`location_url` text,
	`file_size_bytes` integer
);
--> statement-breakpoint
ALTER TABLE `haex_extensions` ADD `haex_tombstone` integer;--> statement-breakpoint
ALTER TABLE `haex_extensions_permissions` ADD `haex_tombstone` integer;--> statement-breakpoint
ALTER TABLE `haex_notifications` ADD `haex_tombstone` integer;--> statement-breakpoint
ALTER TABLE `haex_passwords_group_items` ADD `haex_tombstone` integer;--> statement-breakpoint
ALTER TABLE `haex_passwords_groups` ADD `haex_tombstone` integer;--> statement-breakpoint
ALTER TABLE `haex_passwords_item_details` ADD `haex_tombstone` integer;--> statement-breakpoint
ALTER TABLE `haex_passwords_item_history` ADD `haex_tombstone` integer;--> statement-breakpoint
ALTER TABLE `haex_passwords_item_key_values` ADD `haex_tombstone` integer;--> statement-breakpoint
ALTER TABLE `haex_settings` ADD `haex_tombstone` integer;