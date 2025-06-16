ALTER TABLE `haex_passwords_items` RENAME TO `haex_passwords_item_details`;--> statement-breakpoint
ALTER TABLE `haex_passwords_items_key_values` RENAME TO `haex_passwords_item_key_values`;--> statement-breakpoint
PRAGMA foreign_keys=OFF;--> statement-breakpoint
CREATE TABLE `__new_haex_passwords_item_key_values` (
	`id` text PRIMARY KEY NOT NULL,
	`item_id` text,
	`key` text,
	`value` text,
	`updated_at` integer,
	FOREIGN KEY (`item_id`) REFERENCES `haex_passwords_item_details`(`id`) ON UPDATE no action ON DELETE no action
);
--> statement-breakpoint
INSERT INTO `__new_haex_passwords_item_key_values`("id", "item_id", "key", "value", "updated_at") SELECT "id", "item_id", "key", "value", "updated_at" FROM `haex_passwords_item_key_values`;--> statement-breakpoint
DROP TABLE `haex_passwords_item_key_values`;--> statement-breakpoint
ALTER TABLE `__new_haex_passwords_item_key_values` RENAME TO `haex_passwords_item_key_values`;--> statement-breakpoint
PRAGMA foreign_keys=ON;--> statement-breakpoint
CREATE TABLE `__new_haex_passwords_group_items` (
	`group_id` text,
	`item_id` text,
	PRIMARY KEY(`item_id`, `group_id`),
	FOREIGN KEY (`group_id`) REFERENCES `haex_passwords_groups`(`id`) ON UPDATE no action ON DELETE no action,
	FOREIGN KEY (`item_id`) REFERENCES `haex_passwords_item_details`(`id`) ON UPDATE no action ON DELETE no action
);
--> statement-breakpoint
INSERT INTO `__new_haex_passwords_group_items`("group_id", "item_id") SELECT "group_id", "item_id" FROM `haex_passwords_group_items`;--> statement-breakpoint
DROP TABLE `haex_passwords_group_items`;--> statement-breakpoint
ALTER TABLE `__new_haex_passwords_group_items` RENAME TO `haex_passwords_group_items`;--> statement-breakpoint
CREATE TABLE `__new_haex_passwords_item_history` (
	`id` text PRIMARY KEY NOT NULL,
	`item_id` text,
	`changed_property` text,
	`old_value` text,
	`new_value` text,
	`created_at` text DEFAULT (CURRENT_TIMESTAMP),
	FOREIGN KEY (`item_id`) REFERENCES `haex_passwords_item_details`(`id`) ON UPDATE no action ON DELETE no action
);
--> statement-breakpoint
INSERT INTO `__new_haex_passwords_item_history`("id", "item_id", "changed_property", "old_value", "new_value", "created_at") SELECT "id", "item_id", "changed_property", "old_value", "new_value", "created_at" FROM `haex_passwords_item_history`;--> statement-breakpoint
DROP TABLE `haex_passwords_item_history`;--> statement-breakpoint
ALTER TABLE `__new_haex_passwords_item_history` RENAME TO `haex_passwords_item_history`;