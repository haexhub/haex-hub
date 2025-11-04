CREATE TABLE `haex_devices` (
	`id` text PRIMARY KEY NOT NULL,
	`device_id` text NOT NULL,
	`name` text NOT NULL,
	`created_at` text DEFAULT (CURRENT_TIMESTAMP),
	`updated_at` integer,
	`haex_timestamp` text
);
--> statement-breakpoint
CREATE UNIQUE INDEX `haex_devices_device_id_unique` ON `haex_devices` (`device_id`);--> statement-breakpoint
DROP INDEX `haex_settings_key_type_value_unique`;--> statement-breakpoint
ALTER TABLE `haex_settings` ADD `device_id` text REFERENCES haex_devices(id);--> statement-breakpoint
CREATE UNIQUE INDEX `haex_settings_device_id_key_type_unique` ON `haex_settings` (`device_id`,`key`,`type`);