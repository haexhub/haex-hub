CREATE TABLE `haex_passwords_group_items` (
	`group_id` text,
	`item_id` text,
	PRIMARY KEY(`item_id`, `group_id`),
	FOREIGN KEY (`group_id`) REFERENCES `haex_passwords_groups`(`id`) ON UPDATE no action ON DELETE no action,
	FOREIGN KEY (`item_id`) REFERENCES `haex_passwords_items`(`id`) ON UPDATE no action ON DELETE no action
);
--> statement-breakpoint
CREATE TABLE `haex_passwords_groups` (
	`id` text PRIMARY KEY NOT NULL,
	`name` text,
	`icon` text,
	`order` integer,
	`color` text,
	`parent_id` text,
	FOREIGN KEY (`parent_id`) REFERENCES `haex_passwords_groups`(`id`) ON UPDATE no action ON DELETE no action
);
--> statement-breakpoint
CREATE TABLE `haex_passwords_item_history` (
	`id` text PRIMARY KEY NOT NULL,
	`item_id` text,
	`changed_property` text,
	`old_value` text,
	`new_value` text,
	`created_at` text DEFAULT (CURRENT_TIMESTAMP),
	FOREIGN KEY (`item_id`) REFERENCES `haex_passwords_items`(`id`) ON UPDATE no action ON DELETE no action
);
--> statement-breakpoint
CREATE TABLE `haex_passwords_items` (
	`id` text PRIMARY KEY NOT NULL,
	`title` text,
	`username` text,
	`password` text,
	`note` text,
	`icon` text,
	`tags` text,
	`url` text,
	`created_at` text DEFAULT (CURRENT_TIMESTAMP),
	`updated_at` integer
);
--> statement-breakpoint
CREATE TABLE `haex_passwords_items_key_values` (
	`id` text PRIMARY KEY NOT NULL,
	`item_id` text,
	`key` text,
	`value` text,
	FOREIGN KEY (`item_id`) REFERENCES `haex_passwords_items`(`id`) ON UPDATE no action ON DELETE no action
);
--> statement-breakpoint
ALTER TABLE `haex_notifications` ADD `source` text;