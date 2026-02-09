import { createFileRoute } from '@tanstack/react-router'
import { gql } from '@apollo/client';
import { useQuery, useMutation } from '@apollo/client/react';
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useState } from 'react';
import type { QueryRoot, MutationRoot, LibraryMutationCreateLibraryArgs } from '../gql';

const GET_LIBRARIES = gql`
  query GetLibraries {
    library {
      libraries {
        id
        name
        description
        size
      }
    }
  }
`;

const CREATE_LIBRARY = gql`
  mutation CreateLibrary($name: String!, $rootPath: String!) {
    library {
      createLibrary(name: $name, rootPath: $rootPath) {
        id
        name
      }
    }
  }
`;

export const Route = createFileRoute('/')({
  component: App,
})

function App() {
  const { data, loading, error, refetch } = useQuery<QueryRoot>(GET_LIBRARIES);
  const [createLibrary] = useMutation<MutationRoot, LibraryMutationCreateLibraryArgs>(CREATE_LIBRARY);

  const [name, setName] = useState('');
  const [rootPath, setRootPath] = useState('');

  const handleCreate = async (e: React.FormEvent) => {
      e.preventDefault();
      try {
          await createLibrary({ variables: { name, rootPath } });
          refetch();
          setName('');
          setRootPath('');
      } catch (err) {
          console.error(err);
      }
  };

  if (loading) return <div className="p-8">Loading...</div>;
  if (error) return <div className="p-8 text-red-500">Error: {error.message}</div>;

  return (
    <div className="p-8 space-y-8">
      <div className="flex justify-between items-center">
        <h1 className="text-3xl font-bold">Beam Media Server</h1>
      </div>

      <div className="space-y-4">
        <h2 className="text-2xl font-semibold">Libraries</h2>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {data?.library?.libraries?.map((lib: any) => (
            <div key={lib.id} className="p-6 border rounded-lg bg-card text-card-foreground shadow-sm hover:shadow-md transition-shadow">
              <h3 className="font-semibold text-lg">{lib.name}</h3>
              <p className="text-muted-foreground">{lib.description || "No description"}</p>
              <p className="text-sm mt-2 text-gray-500">Items: {lib.size}</p>
            </div>
          ))}
          {data?.library?.libraries?.length === 0 && (
             <div className="p-8 text-center border border-dashed rounded-lg text-muted-foreground col-span-full">
                No libraries found. Create one below to get started.
             </div>
          )}
        </div>
      </div>

      <div className="max-w-md space-y-6 pt-6 border-t">
         <h2 className="text-xl font-semibold">Create Library</h2>
         <form onSubmit={handleCreate} className="space-y-4">
            <div className="space-y-2">
                <Label htmlFor="name">Name</Label>
                <Input id="name" value={name} onChange={(e) => setName(e.target.value)} placeholder="e.g. Movies" required />
            </div>
            <div className="space-y-2">
                <Label htmlFor="rootPath">Root Path</Label>
                <Input id="rootPath" value={rootPath} onChange={(e) => setRootPath(e.target.value)} placeholder="e.g. /media/movies" required />
                <p className="text-xs text-muted-foreground">Absolute path on the server.</p>
            </div>
            <Button type="submit">Create Library</Button>
         </form>
      </div>
    </div>
  )
}
