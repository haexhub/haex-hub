PRAGMA foreign_keys=OFF;--> statement-breakpoint
CREATE TABLE `__new_haex_desktop_items` (
	`id` text PRIMARY KEY NOT NULL,
	`workspace_id` text NOT NULL,
	`item_type` text NOT NULL,
	`reference_id` text NOT NULL,
	`position_x` integer DEFAULT 0 NOT NULL,
	`position_y` integer DEFAULT 0 NOT NULL,
	`haex_tombstone` integer,
	`haex_timestamp` text,
	FOREIGN KEY (`workspace_id`) REFERENCES `haex_workspaces`(`id`) ON UPDATE no action ON DELETE cascade
);
--> statement-breakpoint
INSERT INTO `__new_haex_desktop_items`("id", "workspace_id", "item_type", "reference_id", "position_x", "position_y", "haex_tombstone", "haex_timestamp") SELECT "id", "workspace_id", "item_type", "reference_id", "position_x", "position_y", "haex_tombstone", "haex_timestamp" FROM `haex_desktop_items`;--> statement-breakpoint
DROP TABLE `haex_desktop_items`;--> statement-breakpoint
ALTER TABLE `__new_haex_desktop_items` RENAME TO `haex_desktop_items`;--> statement-breakpoint
PRAGMA foreign_keys=ON;--> statement-breakpoint
CREATE TABLE `__new_haex_extension_permissions` (
	`id` text PRIMARY KEY NOT NULL,
	`extension_id` text NOT NULL,
	`resource_type` text,
	`action` text,
	`target` text,
	`constraints` text,
	`status` text DEFAULT 'denied' NOT NULL,
	`created_at` text DEFAULT (CURRENT_TIMESTAMP),
	`updated_at` integer,
	`haex_tombstone` integer,
	`haex_timestamp` text,
	FOREIGN KEY (`extension_id`) REFERENCES `haex_extensions`(`id`) ON UPDATE no action ON DELETE cascade
);
--> statement-breakpoint
INSERT INTO `__new_haex_extension_permissions`("id", "extension_id", "resource_type", "action", "target", "constraints", "status", "created_at", "updated_at", "haex_tombstone", "haex_timestamp") SELECT "id", "extension_id", "resource_type", "action", "target", "constraints", "status", "created_at", "updated_at", "haex_tombstone", "haex_timestamp" FROM `haex_extension_permissions`;--> statement-breakpoint
DROP TABLE `haex_extension_permissions`;--> statement-breakpoint
ALTER TABLE `__new_haex_extension_permissions` RENAME TO `haex_extension_permissions`;--> statement-breakpoint
CREATE UNIQUE INDEX `haex_extension_permissions_extension_id_resource_type_action_target_unique` ON `haex_extension_permissions` (`extension_id`,`resource_type`,`action`,`target`);--> statement-breakpoint
CREATE TABLE `__new_haex_passwords_group_items` (
	`group_id` text NOT NULL,
	`item_id` text NOT NULL,
	`haex_tombstone` integer,
	PRIMARY KEY(`item_id`, `group_id`),
	FOREIGN KEY (`group_id`) REFERENCES `haex_passwords_groups`(`id`) ON UPDATE no action ON DELETE cascade,
	FOREIGN KEY (`item_id`) REFERENCES `haex_passwords_item_details`(`id`) ON UPDATE no action ON DELETE cascade
);
--> statement-breakpoint
INSERT INTO `__new_haex_passwords_group_items`("group_id", "item_id", "haex_tombstone") SELECT "group_id", "item_id", "haex_tombstone" FROM `haex_passwords_group_items`;--> statement-breakpoint
DROP TABLE `haex_passwords_group_items`;--> statement-breakpoint
ALTER TABLE `__new_haex_passwords_group_items` RENAME TO `haex_passwords_group_items`;--> statement-breakpoint
CREATE TABLE `__new_haex_passwords_groups` (
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
	FOREIGN KEY (`parent_id`) REFERENCES `haex_passwords_groups`(`id`) ON UPDATE no action ON DELETE cascade
);
--> statement-breakpoint
INSERT INTO `__new_haex_passwords_groups`("id", "name", "description", "icon", "order", "color", "parent_id", "created_at", "updated_at", "haex_tombstone") SELECT "id", "name", "description", "icon", "order", "color", "parent_id", "created_at", "updated_at", "haex_tombstone" FROM `haex_passwords_groups`;--> statement-breakpoint
DROP TABLE `haex_passwords_groups`;--> statement-breakpoint
ALTER TABLE `__new_haex_passwords_groups` RENAME TO `haex_passwords_groups`;--> statement-breakpoint
CREATE TABLE `__new_haex_passwords_item_history` (
	`id` text PRIMARY KEY NOT NULL,
	`item_id` text NOT NULL,
	`changed_property` text,
	`old_value` text,
	`new_value` text,
	`created_at` text DEFAULT (CURRENT_TIMESTAMP),
	`haex_tombstone` integer,
	FOREIGN KEY (`item_id`) REFERENCES `haex_passwords_item_details`(`id`) ON UPDATE no action ON DELETE cascade
);
--> statement-breakpoint
INSERT INTO `__new_haex_passwords_item_history`("id", "item_id", "changed_property", "old_value", "new_value", "created_at", "haex_tombstone") SELECT "id", "item_id", "changed_property", "old_value", "new_value", "created_at", "haex_tombstone" FROM `haex_passwords_item_history`;--> statement-breakpoint
DROP TABLE `haex_passwords_item_history`;--> statement-breakpoint
ALTER TABLE `__new_haex_passwords_item_history` RENAME TO `haex_passwords_item_history`;--> statement-breakpoint
CREATE TABLE `__new_haex_passwords_item_key_values` (
	`id` text PRIMARY KEY NOT NULL,
	`item_id` text NOT NULL,
	`key` text,
	`value` text,
	`updated_at` integer,
	`haex_tombstone` integer,
	FOREIGN KEY (`item_id`) REFERENCES `haex_passwords_item_details`(`id`) ON UPDATE no action ON DELETE cascade
);
--> statement-breakpoint
INSERT INTO `__new_haex_passwords_item_key_values`("id", "item_id", "key", "value", "updated_at", "haex_tombstone") SELECT "id", "item_id", "key", "value", "updated_at", "haex_tombstone" FROM `haex_passwords_item_key_values`;--> statement-breakpoint
DROP TABLE `haex_passwords_item_key_values`;--> statement-breakpoint
ALTER TABLE `__new_haex_passwords_item_key_values` RENAME TO `haex_passwords_item_key_values`;