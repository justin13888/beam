import Elysia from "elysia";

const profile = new Elysia({ prefix: "/profile" });
// .put('profile', () => "") // TODO: Update profile info (username, email, name, etc.)
// .put('profile/avatar', () => "") // TODO: Update profile avatar
// .get('profile/avatar', () => "") // TODO: Get profile avatar
// .get('profile', () => "") // TODO: Get profile info
// .get('profile/:username', () => "") // TODO: Get profile info by username
// .get('profile/:username/avatar', () => "") // TODO: Get profile avatar by username/
// TODO: Add endpoints to update any profile info
export default profile;
