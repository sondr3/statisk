module Statisk
  module CLI
    module New
      class Post
        def initialize(options = {})
          @title = options[:title]

          @slug = Helpers.slugify(@title)
          @name = Helpers.titleize(@title)
          puts "Created post with title #{@name}"

          create
          write_post
        end

        private
        def file_path
          "content/posts/#{Time.now.strftime("%F")}-#{@slug}.md"
        end

        def create
          FileUtils.touch(file_path)
        end

        def metadata
          <<-METADATA.gsub /^\s+/, ''
            ---
            title: #{@name}
            slug: #{@slug}
            date: #{Time.now.utc.iso8601}
            ---
          METADATA
        end

        def write_post
          File.write(file_path, metadata)
        end
      end

      class Page
        def initialize(options = {})
          @title = options[:title]

          @slug = Helpers.slugify(@title)
          @name = Helpers.titleize(@title)
          puts "Created page with the title #{@name}"

          create
          write_page
        end

        private
        def file_path
          "content/#{@slug}.md"
        end

        def create
          FileUtils.touch(file_path)
        end

        def metadata
          <<-METADATA.gsub /^\s+/, ''
            ---
            title: #{@name}
            slug: #{@slug}
            date: #{Time.now.utc.iso8601}
            ---
          METADATA
        end

        def write_page
          File.write(file_path, metadata)
        end
      end

      class Draft
        def initialize(options = {})
          @title = options[:title]

          @slug = Helpers.slugify(@title)
          @name = Helpers.titleize(@title)

          puts "Created draft with the title #{@name}"

          create
          write_draft
        end

        private
        def file_path
          "content/drafts/#{Time.now.strftime("%F")}-#{@slug}.md"
        end

        def create
          FileUtils.touch(file_path)
        end

        def metadata
          <<-METADATA.gsub /^\s+/, ''
            ---
            title: #{@name}
            slug: #{@slug}
            date: #{Time.now.utc.iso8601}
            ---
          METADATA
        end

        def write_draft
          File.write(file_path, metadata)
        end
      end
    end
  end
end
