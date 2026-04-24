import { Nav } from '@/components/nav';
import { ScrollProgress } from '@/components/scroll-progress';
import { Hero } from '@/components/hero';
import { Features } from '@/components/features';
import { Pipeline } from '@/components/pipeline';
import { Bento } from '@/components/bento';
import { Testimonial } from '@/components/testimonial';
import { Stack } from '@/components/stack';
import { Security } from '@/components/security';
import { Faq } from '@/components/faq';
import { CTA } from '@/components/cta';
import { Footer } from '@/components/footer';

export default function HomePage() {
  return (
    <>
      <ScrollProgress />
      <Nav />
      <main className="relative">
        <Hero />
        <Features />
        <Pipeline />
        <Bento />
        <Testimonial />
        <Stack />
        <Security />
        <Faq />
        <CTA />
      </main>
      <Footer />
    </>
  );
}
