ALTER TABLE `haex_extension_permissions` RENAME COLUMN "resource" TO "resource_type";--> statement-breakpoint
ALTER TABLE `haex_extension_permissions` RENAME COLUMN "operation" TO "action";--> statement-breakpoint
ALTER TABLE `haex_extension_permissions` RENAME COLUMN "path" TO "target";--> statement-breakpoint
DROP INDEX `haex_extension_permissions_extension_id_resource_operation_path_unique`;--> statement-breakpoint
ALTER TABLE `haex_extension_permissions` ADD `constraints` text;--> statement-breakpoint
ALTER TABLE `haex_extension_permissions` ADD `status` text DEFAULT 'denied' NOT NULL;--> statement-breakpoint
ALTER TABLE `haex_extension_permissions` ADD `haex_timestamp` text;--> statement-breakpoint
CREATE UNIQUE INDEX `haex_extension_permissions_extension_id_resource_type_action_target_unique` ON `haex_extension_permissions` (`extension_id`,`resource_type`,`action`,`target`);--> statement-breakpoint
ALTER TABLE `haex_extensions` ADD `description` text;--> statement-breakpoint
ALTER TABLE `haex_extensions` ADD `entry` text;--> statement-breakpoint
ALTER TABLE `haex_extensions` ADD `homepage` text;--> statement-breakpoint
ALTER TABLE `haex_extensions` ADD `public_key` text;--> statement-breakpoint
ALTER TABLE `haex_extensions` ADD `signature` text;--> statement-breakpoint
ALTER TABLE `haex_extensions` ADD `haex_timestamp` text;--> statement-breakpoint
ALTER TABLE `haex_settings` ADD `haex_timestamp` text;