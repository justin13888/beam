import { gql, type TypedDocumentNode } from "@apollo/client";
import { useQuery } from "@apollo/client/react";
import { createFileRoute } from "@tanstack/react-router";
import type {
	GetMediaMetadataByIdQuery,
	GetMediaMetadataByIdQueryVariables,
} from "@/gql";

const GET_METADATA_BY_ID: TypedDocumentNode<
	GetMediaMetadataByIdQuery,
	GetMediaMetadataByIdQueryVariables
> = gql`
  	query GetMediaMetadataById($mediaId: ID!) {
		media {
			metadata(id: $mediaId) {
			__typename
			... on ShowMetadata {
				title {
				original
				localized
				alternatives
				}
				description
				year
				seasons {
					seasonNumber
					dates {
						firstAired
						lastAired
					}
					episodeRuntime
					posterUrl
					genres
					ratings {
						tmdb
					}
					identifiers {
						imdbId
						tmdbId
						tvdbId
					}
				}
			}
			... on MovieMetadata {
				title {
				original
				localized
				alternatives
				}
				description
				year
				releaseDate
				runtime
				duration
				posterUrl
				backdropUrl
				genres
				ratings {
				tmdb
				}
				identifiers {
				imdbId
				tmdbId
				tvdbId
				}
			}
			}
		}
	}
`;

export const Route = createFileRoute("/media/$id")({
	component: RouteComponent,
});

function RouteComponent() {
	const { id } = Route.useParams();
	const { loading, error, data } = useQuery(GET_METADATA_BY_ID, {
		variables: { mediaId: id },
	});

	if (loading) return <p>Loading...</p>;
	if (error) return <p>Error : {error.message}</p>;

	return (
		<div>
			<pre>{JSON.stringify(data, null, 2)}</pre>
		</div>
	);
}
