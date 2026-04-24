import { Nav } from '@/components/nav';
import { Hero } from '@/components/hero';
import { Marquee } from '@/components/marquee';
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
      <Nav />
      <main className="relative">
        <Hero />
        <Marquee />
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
