import {
    USERNAME_MAX_LENGTH,
    DATABASE_PREFIX as prefix,
} from "@/lib/constants";
import { relations } from "drizzle-orm";
import {
    boolean,
    index,
    pgEnum,
    pgTableCreator,
    serial,
    text,
    timestamp,
    uuid,
    varchar,
} from "drizzle-orm/pg-core";

export const pgTable = pgTableCreator((name) => `${prefix}_${name}`);

export const accountStatusEnum = pgEnum("account_status", [
    "active",
    "inactive",
    "banned",
]);
// TODO: Implement email verification
export const users = pgTable("users", {
    username: varchar("username", { length: USERNAME_MAX_LENGTH }).primaryKey(),
    email: varchar("email", { length: 255 }).unique().notNull(),
    status: accountStatusEnum("status").default("active").notNull(),
    hashedPassword: varchar("hashed_password", { length: 255 }).notNull(),
    createdAt: timestamp("created_at").defaultNow().notNull(),
    updatedAt: timestamp("updated_at").$onUpdate(() => new Date()),
    isAdmin: boolean("is_admin").default(false).notNull(),
});

export type User = typeof users.$inferSelect;
export type NewUser = typeof users.$inferInsert;

export const userRelations = relations(users, ({ one, many }) => ({
    profile: one(profiles),
    sessions: many(sessions),
}));

export const profiles = pgTable("profiles", {
    id: uuid("id").defaultRandom().primaryKey(),
    username: varchar("username", { length: USERNAME_MAX_LENGTH })
        .unique()
        .notNull()
        .references(() => users.username),
    fullname: varchar("fullname", { length: 255 }).notNull(),
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

export const osEnum = pgEnum("os", [
    "windows",
    "mac",
    "linux",
    "android",
    "ios",
    "other",
]);
export const loginMethodEnum = pgEnum("login_method", ["password", "qr_code"]);

export const sessions = pgTable(
    "sessions",
    {
        id: varchar("id", { length: 255 }).primaryKey(),
        createdAt: timestamp("created_at").defaultNow().notNull(),
        expiresAt: timestamp("expires_at").notNull(),
        lastUsedAt: timestamp("last_used_at").notNull(),
        username: varchar("username", { length: 255 }).notNull(),
        deviceName: varchar("device_name", { length: 255 }).notNull(),
        os: osEnum("os").notNull(),
        ip: text("ip").notNull(), // TODO: Change to better type
        loginMethod: loginMethodEnum("login_method").notNull(),
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

export const loginHistory = pgTable(
    "login_history",
    {
        id: serial("id").primaryKey(),
        username: varchar("username", { length: 255 }).notNull(),
        timestamp: timestamp("timestamp").notNull().defaultNow(),
        ip: text("ip").notNull(), // TODO: Change to better type
        os: osEnum("os").notNull(),
        deviceName: varchar("device_name", { length: 255 }).notNull(),
    },
    (t) => ({
        timestampIdx: index("timestamp_idx").on(t.timestamp),
    }),
);

export type LoginHistory = typeof loginHistory.$inferSelect;

export const loginHistoryRelations = relations(loginHistory, ({ one }) => ({
    user: one(users, {
        fields: [loginHistory.username],
        references: [users.username],
    }),
}));

export const passwordResetTokens = pgTable(
    "password_reset_tokens",
    {
        id: uuid("id").defaultRandom().primaryKey(),
        token: varchar("token", { length: 255 }).notNull(),
        userId: varchar("user_id", { length: 21 }).notNull(),
        expiresAt: timestamp("expires_at").notNull(),
    },
    (t) => ({
        tokenIdx: index("token_idx").on(t.token),
        // userIdx: index("user_idx").on(t.userId),
    }),
);

export type PasswordResetToken = typeof passwordResetTokens.$inferSelect;

export const passwordResetTokenRelations = relations(
    passwordResetTokens,
    ({ one }) => ({
        user: one(users, {
            fields: [passwordResetTokens.userId],
            references: [users.username],
        }),
    }),
);

export const collections = pgTable(
    "collections",
    {
        // ID should only contain lowercase, uppercase, and numbers
        id: varchar("id", { length: 11 }).primaryKey(),
        name: varchar("name", { length: 255 }).notNull(),
        description: text("description"),
        createdAt: timestamp("created_at").defaultNow().notNull(),
        updatedAt: timestamp("updated_at").$onUpdate(() => new Date()),
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

export const media = pgTable(
    "media",
    {
        // ID should only contain lowercase, uppercase, and numbers
        id: varchar("id", { length: 15 }).primaryKey(),
        collectionId: varchar("collection_id", { length: 11 }).references(
            () => collections.id,
        ),
        name: varchar("name", { length: 255 }).notNull(),
        description: text("description"),
        type: varchar("type", {
            length: 10,
            enum: ["image", "video"],
        }).notNull(),
        createdAt: timestamp("created_at").defaultNow().notNull(),
        updatedAt: timestamp("updated_at").$onUpdate(() => new Date()),
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
