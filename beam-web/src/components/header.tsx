import { Link } from "@tanstack/react-router";
import { ModeToggle } from "./mode-toggle";

export const Header = () => {
  return (
    <>
    <div className="p-2 flex gap-2">
      <Link to="/" className="[&.active]:font-bold">
        Home
      </Link>{" "}
      <Link to="/about" className="[&.active]:font-bold">
        About
      </Link>
    </div>

    <ModeToggle />
    </>
  );
};

// TODO: Implement navlinks, profile
