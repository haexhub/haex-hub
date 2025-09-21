ALTER TABLE `haex_crdt_settings` RENAME TO `haex_crdt_configs`;--> statement-breakpoint
ALTER TABLE `haex_extensions_permissions` RENAME TO `haex_extension_permissions`;--> statement-breakpoint
ALTER TABLE `haex_crdt_configs` RENAME COLUMN "type" TO "key";--> statement-breakpoint
PRAGMA foreign_keys=OFF;--> statement-breakpoint
CREATE TABLE `__new_haex_extension_permissions` (
	`id` text PRIMARY KEY NOT NULL,
	`extension_id` text,
	`resource` text,
	`operation` text,
	`path` text,
	`created_at` text DEFAULT (CURRENT_TIMESTAMP),
	`updated_at` integer,
	`haex_tombstone` integer,
	FOREIGN KEY (`extension_id`) REFERENCES `haex_extensions`(`id`) ON UPDATE no action ON DELETE no action
);
--> statement-breakpoint
INSERT INTO `__new_haex_extension_permissions`("id", "extension_id", "resource", "operation", "path", "created_at", "updated_at", "haex_tombstone") SELECT "id", "extension_id", "resource", "operation", "path", "created_at", "updated_at", "haex_tombstone" FROM `haex_extension_permissions`;--> statement-breakpoint
DROP TABLE `haex_extension_permissions`;--> statement-breakpoint
ALTER TABLE `__new_haex_extension_permissions` RENAME TO `haex_extension_permissions`;--> statement-breakpoint
PRAGMA foreign_keys=ON;--> statement-breakpoint
CREATE UNIQUE INDEX `haex_extension_permissions_extension_id_resource_operation_path_unique` ON `haex_extension_permissions` (`extension_id`,`resource`,`operation`,`path`);