DROP INDEX `haex_workspaces_name_unique`;--> statement-breakpoint
CREATE UNIQUE INDEX `haex_workspaces_position_unique` ON `haex_workspaces` (`position`);--> statement-breakpoint
CREATE UNIQUE INDEX `haex_settings_key_type_value_unique` ON `haex_settings` (`key`,`type`,`value`);