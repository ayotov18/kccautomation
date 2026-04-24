import { redirect } from 'next/navigation';

/**
 * Legacy route: `/upload` was its own page. The new IA surfaces upload as a
 * button on the Drawings list — no need for a standalone route. Preserve old
 * links by redirecting.
 */
export default function UploadRedirect() {
  redirect('/drawings?upload=1');
}
