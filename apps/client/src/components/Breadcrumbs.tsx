interface BreadcrumbItem {
  id: string;
  title: string;
  path: string[];
}

interface BreadcrumbsProps {
  crumbs: BreadcrumbItem[];
  onNavigate: (path: string[]) => void;
}

export function Breadcrumbs({ crumbs, onNavigate }: BreadcrumbsProps) {
  return (
    <nav className="breadcrumbs" aria-label="Breadcrumb navigation">
      {crumbs.map((crumb, index) => {
        const isLast = index === crumbs.length - 1;

        return (
          <span key={crumb.id}>
            {index > 0 && <span className="separator"> / </span>}
            {isLast ? (
              <strong className="current">{crumb.title}</strong>
            ) : (
              <button
                className="crumb-link"
                onClick={() => {
                  onNavigate(crumb.path);
                }}
              >
                {crumb.title}
              </button>
            )}
          </span>
        );
      })}
    </nav>
  );
}
