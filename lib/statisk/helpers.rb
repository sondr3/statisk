module Statisk
  class Helpers
    def self.slugify(title)
      @title = title

      slug = @title.downcase.strip.gsub(' ', '-').gsub(/[^\w-]/, '')
    end

    def self.titleize(title)
      @title = title

      @title.capitalize
    end
  end
end
