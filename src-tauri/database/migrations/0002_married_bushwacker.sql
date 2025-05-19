ALTER TABLE `haex_settings` RENAME COLUMN "value_text" TO "value";--> statement-breakpoint
DROP TABLE `testTable`;--> statement-breakpoint
ALTER TABLE `haex_settings` DROP COLUMN `value_json`;--> statement-breakpoint
ALTER TABLE `haex_settings` DROP COLUMN `value_number`;