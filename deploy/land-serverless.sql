# ************************************************************
# Sequel Ace SQL dump
# Version 20046
#
# https://sequel-ace.com/
# https://github.com/Sequel-Ace/Sequel-Ace
#
# Host: 127.0.0.1 (MySQL 5.7.39)
# Database: lol-serverless
# Generation Time: 2023-04-26 22:50:53 +0000
# ************************************************************


/*!40101 SET @OLD_CHARACTER_SET_CLIENT=@@CHARACTER_SET_CLIENT */;
/*!40101 SET @OLD_CHARACTER_SET_RESULTS=@@CHARACTER_SET_RESULTS */;
/*!40101 SET @OLD_COLLATION_CONNECTION=@@COLLATION_CONNECTION */;
SET NAMES utf8mb4;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE='NO_AUTO_VALUE_ON_ZERO', SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;


# Dump of table project_deployment
# ------------------------------------------------------------

DROP TABLE IF EXISTS `project_deployment`;

CREATE TABLE `project_deployment` (
  `id` int(11) unsigned NOT NULL AUTO_INCREMENT,
  `owner_id` int(11) NOT NULL COMMENT 'project id',
  `project_id` int(11) NOT NULL COMMENT 'project id',
  `domain` varchar(64) NOT NULL,
  `prod_domain` varchar(64) NOT NULL,
  `uuid` varchar(128) NOT NULL,
  `storage_path` varchar(128) NOT NULL,
  `created_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  `prod_status` int(11) NOT NULL DEFAULT '0',
  `deploy_status` int(11) NOT NULL DEFAULT '0',
  PRIMARY KEY (`id`),
  UNIQUE KEY `dev_domain` (`domain`),
  KEY `project_id` (`project_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;



# Dump of table project_info
# ------------------------------------------------------------

DROP TABLE IF EXISTS `project_info`;

CREATE TABLE `project_info` (
  `id` int(11) unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(64) NOT NULL COMMENT 'project name',
  `language` varchar(24) NOT NULL COMMENT 'project language',
  `uuid` varchar(64) NOT NULL COMMENT 'project description',
  `created_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'project created time',
  `updated_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'project updated time',
  `owner_id` int(11) DEFAULT '0',
  `prod_deploy_id` int(11) DEFAULT '0' COMMENT 'production deployment id',
  PRIMARY KEY (`id`),
  UNIQUE KEY `name` (`name`),
  KEY `owner_id` (`owner_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;


# Dump of table user_info
# ------------------------------------------------------------

DROP TABLE IF EXISTS `user_info`;

CREATE TABLE `user_info` (
  `id` int(11) unsigned NOT NULL AUTO_INCREMENT,
  `email` varchar(128) NOT NULL COMMENT 'user email',
  `password` varchar(128) NOT NULL COMMENT 'user password',
  `password_salt` varchar(64) NOT NULL COMMENT 'user password salt',
  `created_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  `display_name` varchar(128) NOT NULL COMMENT 'user display name in webpage',
  `role` int(11) NOT NULL DEFAULT '2' COMMENT 'user role, admin=1/user=2',
  PRIMARY KEY (`id`),
  UNIQUE KEY `email` (`email`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;


# Dump of table user_token
# ------------------------------------------------------------

DROP TABLE IF EXISTS `user_token`;

CREATE TABLE `user_token` (
  `id` int(11) unsigned NOT NULL AUTO_INCREMENT,
  `owner_id` int(11) NOT NULL COMMENT 'token owner',
  `token` varchar(128) NOT NULL,
  `uuid` varchar(64) NOT NULL,
  `name` varchar(64) NOT NULL,
  `created_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  `origin` varchar(24) NOT NULL,
  `expired_at` int(16) NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uuid` (`uuid`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

# Mock data for table user_info
# ------------------------------------------------------------

INSERT INTO `user_info` (`email`, `password`, `password_salt`, `created_at`, `updated_at`, `display_name`, `role`)
VALUES
	('abc@abc.com', '$2b$12$FqoBc8isppfw6aLY28zPy.xpDm.U21TRQgkoKYPaOFQsJV09hOBUy', 'g4L0JdxhOG', '2023-04-27 13:45:04', '2023-05-29 15:16:35', 'batman', 1);

INSERT INTO `user_token` (`owner_id`, `token`, `uuid`, `name`, `created_at`, `updated_at`, `origin`, `expired_at`)
VALUES
	(1, 'b5bddtoxjjnT7yS9Mm1ngDoscveYeWetGCff4xk8', 'fb366c76-def9-4161-bf06-8621b497275f', 'cli-test', '2023-05-18 10:42:12', '2023-06-06 16:18:42', 'dashboard', 1715913732);


/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;
/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40101 SET CHARACTER_SET_CLIENT=@OLD_CHARACTER_SET_CLIENT */;
/*!40101 SET CHARACTER_SET_RESULTS=@OLD_CHARACTER_SET_RESULTS */;
/*!40101 SET COLLATION_CONNECTION=@OLD_COLLATION_CONNECTION */;
