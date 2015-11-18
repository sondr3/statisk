module Statisk
  module CLI
    class Init
      def initialize
        puts " Building your site".blue
        scaffold
        posts
      end

      private
      def scaffold
        puts " Creating folders".blue
        Dir.foreach(templates) do |file|
          next if file == "." or file == ".."
          FileUtils.cp_r templates + "/.", Dir.pwd
        end
      end

      def templates
        File.expand_path("../../template", File.dirname(__FILE__))
      end

      def posts
        puts " Creating posts".blue
        File.rename("content/posts/2000-00-00-another-world.md", "content/posts/#{Time.now.strftime("%F")}-another-world.md")
        File.rename("content/drafts/2000-00-00-something-else.md", "content/drafts/#{Time.now.strftime("%F")}-another-world.md")
      end
    end
  end
end
