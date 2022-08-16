CREATE TABLE `item` (
	`id` INT(10) UNSIGNED NOT NULL,
	`name` TEXT NOT NULL COLLATE 'utf8mb3_general_ci',
	`specification` TEXT NULL DEFAULT NULL COLLATE 'utf8mb3_general_ci',
	`unit` TEXT NULL DEFAULT NULL COLLATE 'utf8mb3_general_ci',
	`manufacturer` TEXT NOT NULL COLLATE 'utf8mb3_general_ci',
	`number` INT(11) NOT NULL,
	`price` FLOAT UNSIGNED NOT NULL DEFAULT '0',
	`expiration` DATE NOT NULL,
	PRIMARY KEY (`id`) USING BTREE
)
COLLATE='utf8mb3_general_ci'
ENGINE=InnoDB
;

CREATE TABLE `batch` (
	`id` INT(10) UNSIGNED NOT NULL,
	`date` DATE NOT NULL,
	`number` INT(11) NOT NULL,
	`expiration` DATE NOT NULL,
	`vendor` TEXT NULL DEFAULT NULL COLLATE 'utf8mb3_general_ci',
	`item_id` INT(10) UNSIGNED NOT NULL,
	PRIMARY KEY (`id`) USING BTREE,
	INDEX `fk_batch_item` (`item_id`) USING BTREE,
	CONSTRAINT `fk_batch_item` FOREIGN KEY (`item_id`) REFERENCES `stocker-vue`.`item` (`id`) ON UPDATE NO ACTION ON DELETE NO ACTION
)
COLLATE='utf8mb3_general_ci'
ENGINE=InnoDB
;

CREATE TABLE `stock_out` (
	`id` INT(10) UNSIGNED NOT NULL,
	`date` DATE NOT NULL,
	`number` INT(11) NOT NULL,
	`item_id` INT(10) UNSIGNED NOT NULL,
	PRIMARY KEY (`id`) USING BTREE,
	INDEX `fk_stock_out_item` (`item_id`) USING BTREE,
	CONSTRAINT `fk_stock_out_item` FOREIGN KEY (`item_id`) REFERENCES `stocker-vue`.`item` (`id`) ON UPDATE NO ACTION ON DELETE NO ACTION
)
COLLATE='utf8mb3_general_ci'
ENGINE=InnoDB
;
