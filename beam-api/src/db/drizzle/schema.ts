import { mysqlEnum, mysqlTableCreator, varbinary } from "drizzle-orm/mysql-core";
import { USERNAME_MAX_LENGTH, DATABASE_PREFIX as prefix } from "@/lib/constants";
import {
  boolean,
  datetime,
  index,
  int,
  text,
  timestamp,
  varchar,
} from "drizzle-orm/mysql-core";
import { relations } from "drizzle-orm";

export const mysqlTable = mysqlTableCreator((name) => `${prefix}_${name}`);
export const users = mysqlTable(
  "users",
  {
    username: varchar("username", { length: 255 }).primaryKey(),
    email: varchar("email", { length: 255 }).unique().notNull(),
    emailVerified: boolean("email_verified").default(false).notNull(), // TODO: Implement email verification
    status: mysqlEnum("status", ["active", "deleted", "banned"]).default("active").notNull(),
    hashed_password: varchar("hashed_password", { length: 255 }).notNull(),
    createdAt: timestamp("created_at").defaultNow().notNull(),
    updatedAt: timestamp("updated_at").onUpdateNow(),
    isAdmin: boolean("is_admin").default(false).notNull(),
  },
  (t) => ({
    emailIdx: index("email_idx").on(t.email),
    usernameIdx: index("username_idx").on(t.username),
  }),
);

export type User = typeof users.$inferSelect;
export type NewUser = typeof users.$inferInsert;

export const userRelations = relations(users, ({ one, many }) => ({
  profile: one(profiles),
  sessions: many(sessions),
}));

export const profiles = mysqlTable('profiles', {
  id: varchar('id', { length: 255 }).primaryKey(),
  username: varchar('username', { length: 255 }).unique().notNull().references(() => users.username),
  fullname: varchar('fullname', { length: 255 }).notNull(),
  // avatar: varchar("avatar", { length: 255 }), // TODO: Review blob or url or base64
});

export type Profile = typeof profiles.$inferSelect;
export type NewProfile = typeof profiles.$inferInsert;

export const profileRelations = relations(profiles, ({ one }) => ({
  user: one(users, {
    fields: [profiles.username],
    references: [users.username],
  }),
}));

export const sessions = mysqlTable(
  "sessions",
  {
    id: varchar("id", { length: 255 }).primaryKey(),
    createdAt: timestamp("created_at").defaultNow().notNull(),
    expiresAt: timestamp("expires_at").notNull(),
    lastUsedAt: timestamp("last_used_at").notNull(),
    username: varchar("username", { length: 255 }).notNull(),
    deviceName: varchar("device_name", { length: 255 }).notNull(),
    os: mysqlEnum("os", ["windows", "mac", "linux", "android", "ios", "other"]).notNull(),
    ip: varbinary("ip", { length: 128 }).notNull(),
    loginMethod: mysqlEnum("login_method", ["password", "qr_code"]).notNull(),
    revoked: boolean("revoked").default(false).notNull(),
  },
  (t) => ({
    usernameIdx: index("username_idx").on(t.username),
    revokedIdx: index("revoked_idx").on(t.revoked),
    expiresAtIdx: index("expires_at_idx").on(t.expiresAt),
  }),
);

export type Session = typeof sessions.$inferSelect;
export type NewSession = typeof sessions.$inferSelect;

export const sessionRelations = relations(sessions, ({ one }) => ({
  user: one(users, {
    fields: [sessions.username],
    references: [users.username],
  }),
}));

export const loginHistory = mysqlTable(
  "login_history",
  {
    id: varchar("id", { length: 40 }).primaryKey(),
    username: varchar("username", { length: 255 }).notNull(),
    timestamp: timestamp("timestamp").notNull().defaultNow(),
    ip: varbinary("ip", { length: 128 }).notNull(),
    os: mysqlEnum("os", ["windows", "mac", "linux", "android", "ios", "other"]).notNull(),
    deviceName: varchar("device_name", { length: 255 }).notNull(),
  },
  (t) => ({
    usernameIdx: index("username_idx").on(t.username),
  }),
);

export type LoginHistory = typeof loginHistory.$inferSelect;

export const loginHistoryRelations = relations(loginHistory, ({ one }) => ({
  user: one(users, {
    fields: [loginHistory.username],
    references: [users.username],
  }),
}));

export const passwordResetTokens = mysqlTable(
  "password_reset_tokens",
  {
    id: varchar("id", { length: 40 }).primaryKey(),
    userId: varchar("user_id", { length: 21 }).notNull(),
    expiresAt: datetime("expires_at").notNull(),
  },
  (t) => ({
    userIdx: index("user_idx").on(t.userId),
  }),
);

export type PasswordResetToken = typeof passwordResetTokens.$inferSelect;

export const passwordResetTokenRelations = relations(passwordResetTokens, ({ one }) => ({
  user: one(users, {
    fields: [passwordResetTokens.userId],
    references: [users.username],
  }),
}));

export const collections = mysqlTable(
  "collections",
  {
    // ID should only contain lowercase, uppercase, and numbers
    id: varchar("id", { length: 11 }).primaryKey(),
    name: varchar("name", { length: 255 }).notNull(),
    description: text("description"),
    createdAt: timestamp("created_at").defaultNow().notNull(),
    updatedAt: timestamp("updated_at").onUpdateNow(),
    // Public collections can be viewed by anyone
    // while private collections are only viewable by the owner
    public: boolean("public").default(false).notNull(),
  },
  (t) => ({
    nameIdx: index("name_idx").on(t.name),
    publicIdx: index("public_idx").on(t.public),
  }),
);

export type NewCollection = typeof collections.$inferSelect;
export type Collection = typeof collections.$inferSelect;
// TODO: Consider implementing role based visibility of collections

export const media = mysqlTable(
  "media",
  {
    // ID should only contain lowercase, uppercase, and numbers
    id: varchar("id", { length: 15 }).primaryKey(),
    collectionId: varchar("collection_id", { length: 11 }).references(() => collections.id),
    name: varchar("name", { length: 255 }).notNull(),
    description: text("description"),
    type: varchar("type", { length: 10, enum: ["image", "video"] }).notNull(),
    createdAt: timestamp("created_at").defaultNow().notNull(),
    updatedAt: timestamp("updated_at").onUpdateNow(),
    // TODO: Implement other metadata based on TMDB api
  },
  (t) => ({
    collectionIdx: index("collection_idx").on(t.collectionId),
  }),
);

export const mediaRelations = relations(media, ({ one }) => ({
  collection: one(collections, {
    fields: [media.collectionId],
    references: [collections.id],
  }),
}));

export type Media = typeof media.$inferSelect;
export type NewMedia = typeof media.$inferInsert;
