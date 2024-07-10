import { ModeToggle } from "@/components/mode-toggle";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuLabel,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";

import {
	NavigationMenu,
	NavigationMenuContent,
	NavigationMenuItem,
	NavigationMenuLink,
	NavigationMenuList,
	NavigationMenuTrigger,
  navigationMenuTriggerStyle,
} from "@/components/ui/navigation-menu";
import { Link } from "@tanstack/react-router";
import { SearchBar } from "./searchbar";

interface Category {
  title: string
  /** Short description */
  description: string
  /** Parameter key */
  key: string
}

const categories: Category[] = [
  {
    title: "Popular",
    description: "The most popular content on the site.",
    key: "popular",
  },
  {
    title: "Now Playing",
    description: "Content that is currently playing.",
    key: "live",
  },
  {
    title: "Upcoming",
    description: "Content that will be available soon.",
    key: "upcoming",
  },
  {
    title: "Recently Added",
    description: "Content that was recently added to the site.",
    key: "recent",
  },
  {
    title: "Trending",
    description: "Content that is currently trending.",
    key: "trending",
  }
];

// TODO: Implement interactivity
export const Header = () => {
	return (
		<>
			<header className="flex items-center justify-between h-16 px-4 md:px-6 bg-background">
				
          {/* Left-aligned elements */}
				<div className="flex items-center space-x-4">
					<Link to="/">
						<img src="/vite.svg" alt="Logo" className="h-8 w-8" />
					</Link>
					<NavigationMenu>
						<NavigationMenuList>
							<NavigationMenuItem>
								<NavigationMenuTrigger>Browse</NavigationMenuTrigger>
								<NavigationMenuContent>
									<ul className="grid gap-3 p-6 md:w-[400px] lg:w-[500px] lg:grid-cols-[.75fr_1fr]">
										<li className="row-span-3">
											<NavigationMenuLink asChild>
												<a
													className="flex h-full w-full select-none flex-col justify-end rounded-md bg-gradient-to-b from-muted/50 to-muted p-6 no-underline outline-none focus:shadow-md"
													href="/"
												>
													{/* <Icons.logo className="h-6 w-6" /> */} 
                          {/* TODO: Get back proper logo */}
                          <img src="/vite.svg" alt="Logo" className="h-6 w-6" />

													<div className="mb-2 mt-4 text-lg font-medium">
														shadcn/ui
													</div>
													<p className="text-sm leading-tight text-muted-foreground">
														Beautifully designed components that you can copy
														and paste into your apps. Accessible. Customizable.
														Open Source.
													</p>
												</a>
											</NavigationMenuLink>
										</li>
                    {
                      categories.map((category) => <ListItem
                        key={category.key}
                        to={`/browse?category=${category.key}`} 
                        title={category.title} 
                        description={category.description} />
                      )
                    }
									</ul>
								</NavigationMenuContent>
							</NavigationMenuItem>
							<NavigationMenuItem>
								<NavigationMenuTrigger>Components</NavigationMenuTrigger>
								<NavigationMenuContent>
									<ul className="grid w-[400px] gap-3 p-4 md:w-[500px] md:grid-cols-2 lg:w-[600px] ">
                  {
                      categories.map((category) => <ListItem
                        key={category.key}
                        to={`/browse?category=${category.key}`} 
                        title={category.title} 
                        description={category.description} />
                      )
                    }
									</ul>
								</NavigationMenuContent>
							</NavigationMenuItem>
							<NavigationMenuItem>
								<Link to="/admin/dashboard">
									<NavigationMenuLink className={navigationMenuTriggerStyle()}>
										Documentation
									</NavigationMenuLink>
								</Link>
							</NavigationMenuItem>
						</NavigationMenuList>
					</NavigationMenu>
				</div>
        {/* Centred elements */}
				<div className="relative flex-1 max-w-md">
          {/* TODO: Position it more centred if possible */}
          <SearchBar />
        </div>

        {/* Right-aligned elements */}
				<div className="flex items-center space-x-4">
					<ModeToggle />
					<DropdownMenu>
						<DropdownMenuTrigger asChild>
							<Avatar className="w-9 h-9 border">
								<AvatarImage src="/placeholder-user.jpg" />
								<AvatarFallback>JP</AvatarFallback>
							</Avatar>
						</DropdownMenuTrigger>
						<DropdownMenuContent align="end">
							<DropdownMenuLabel>My Account</DropdownMenuLabel>
							<DropdownMenuSeparator />
							<DropdownMenuItem>Settings</DropdownMenuItem>
							<DropdownMenuItem>Log out</DropdownMenuItem>
						</DropdownMenuContent>
					</DropdownMenu>
				</div>
        
			</header>
		</>
	);
};

interface ListItemProps {
  title: string
  description: string
  to: string
};

const ListItem = ({title, description, to}: ListItemProps) => {
  return (
    <li>
      <NavigationMenuLink asChild>
        <Link
          to={to}
          className="block select-none space-y-1 rounded-md p-3 leading-none no-underline outline-none transition-colors hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground"
        >
          <div className="text-sm font-medium leading-none">{title}</div>
          <p className="line-clamp-2 text-sm leading-snug text-muted-foreground">
            {description}
          </p>
        </Link>
      </NavigationMenuLink>
    </li>
  );
};

// TODO: Implement navlinks, profile
