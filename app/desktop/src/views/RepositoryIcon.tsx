/** Folder and branch mark used wherever a repository needs a visual identifier. */
export function RepositoryIcon() {
  return (
    <svg
      className="repository-icon"
      width="20"
      height="20"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.8"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      focusable="false"
    >
      <path d="M8 21H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l3 3h8a2 2 0 0 1 2 2v6" />
      <circle cx="13" cy="12" r="2" />
      <path d="M13 14v9m0-8a5 5 0 0 0 5 5" />
      <circle cx="20" cy="20" r="2" />
    </svg>
  );
}
